//! DEPRECATE (dev-onion-mix only): active onion mesh daemon — replaced by UES Monocell Pool.
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use its_transport::field_arith::FieldElement;
use its_transport::onion::{MorphicMixingNode, MorphicOnionPacket};
use its_transport::trapdoor::Trapdoor;
use its_transport::TransportOtpRatchet;
use its_transport::SecureRandom;

use crate::config::Config;
use crate::courier::{PacketCourier, UdpCourier};
use crate::packet::{deserialize_packet, serialize_packet};
use crate::rng;

pub fn run_node(config: Config) {
    let bind_addr = format!("{}:{}", config.node.bind_address, config.node.port);
    let socket = UdpSocket::bind(&bind_addr).expect("Could not bind UDP socket");
    let courier: Arc<dyn PacketCourier + Send + Sync> = Arc::new(UdpCourier::new(socket));
    println!("Morphic Routing Node {} running on {} via abstract PacketCourier", config.node.id, bind_addr);

    // Setup MorphicMixingNode
    let public_point = (FieldElement::new(1), FieldElement::new(8));
    let trapdoor = Trapdoor::<2>::new([
        (FieldElement::new(config.crypto.trapdoor_x), FieldElement::new(config.crypto.trapdoor_y)),
        public_point,
    ]);

    // Initialize TransportOtpRatchet (SSS OTP — no HKDF)
    // Seed is derived from the node's private trapdoor coordinates
    let mut seed = [0u8; 32];
    seed[0..4].copy_from_slice(&config.crypto.trapdoor_x.to_be_bytes());
    seed[4..8].copy_from_slice(&config.crypto.trapdoor_y.to_be_bytes());
    let ratchet = Arc::new(Mutex::new(TransportOtpRatchet::new(seed)));

    // Step the ratchet to get the first set of keys
    let (_k_pool, mac_key, nonce) = ratchet.lock().unwrap().step().expect("Could not initialize ratchet");

    let node = Arc::new(Mutex::new(MorphicMixingNode::new(
        FieldElement::new(config.node.id),
        trapdoor,
        mac_key,
        nonce,
    )));

    // Active packet queue for constant-rate transmission
    let queue = Arc::new(Mutex::new(Vec::<(FieldElement, MorphicOnionPacket)>::new()));

    // 1. RECEIVER TASK (OS Thread)
    let courier_recv = courier.clone();
    let node_recv = node.clone();
    let queue_recv = queue.clone();
    let ratchet_recv = ratchet.clone();
    thread::spawn(move || {
        let mut buf = [0u8; 1024];
        let mut rng = rng::RoutingRng;
        loop {
            if let Ok((len, _src)) = courier_recv.recv_raw(&mut buf) {
                if let Ok(packet) = deserialize_packet(&buf[..len]) {
                    let mut n = node_recv.lock().unwrap();
                    let mut processed = false;
                    for hop_index in 0..3 {
                        if let Ok((next_hop, forwarded_packet)) = n.process_packet(&packet, hop_index, &mut rng) {
                            if next_hop.value() as u32 != 0 {
                                println!("Received valid packet! Next hop ID: {}", next_hop.value() as u32);
                                queue_recv.lock().unwrap().push((next_hop, forwarded_packet));
                                processed = true;

                                // Step the ratchet to rotate keys and nonces for the next packet
                                if let Ok((_, next_mac, next_nonce)) = ratchet_recv.lock().unwrap().step() {
                                    n.header_mac_key = next_mac;
                                    n.header_nonce = next_nonce;
                                    println!("TransportOtpRatchet stepped: keys and nonces rotated.");
                                }
                                break;
                            }
                        }
                    }
                    if !processed {
                        println!("Received invalid or dummy packet. Ignoring.");
                    }
                }
            }
        }
    });

    // 2. CONSTANT-RATE SENDER TASK (OS Thread)
    let courier_send = courier.clone();
    let node_send = node.clone();
    let queue_send = queue.clone();
    let routing_table = config.routing_table.clone();
    let chaff_enabled = config.traffic.constant_rate_chaff_enabled;

    thread::spawn(move || {
        let mut rng = rng::RoutingRng;
        loop {
            // Get delay from Lorenz Attractor
            let delay_ms = node_send.lock().unwrap().get_next_delay_ms();
            thread::sleep(Duration::from_millis(delay_ms as u64));

            let popped = {
                let mut q = queue_send.lock().unwrap();
                if !q.is_empty() {
                    Some(q.remove(0))
                } else {
                    None
                }
            };

            if let Some((next_hop, packet)) = popped {
                let next_hop_val = next_hop.value() as u32;
                if let Some(addr_str) = routing_table.get(&next_hop_val) {
                    let bytes = serialize_packet(&packet);
                    let _ = courier_send.send_raw(&bytes, addr_str);
                    println!("Sent real packet to next hop {} ({})", next_hop_val, addr_str);
                }
            } else if chaff_enabled {
                // Generate and send a dummy packet to maintain constant rate
                let hops = [
                    (FieldElement::new(1), FieldElement::new(8)),
                    (FieldElement::new(2), FieldElement::new(11)),
                    (FieldElement::new(3), FieldElement::new(14)),
                ];
                let dummy_packet = node_send
                    .lock()
                    .unwrap()
                    .pop_constant_rate_packet(&mut rng, hops);
                // Pick a random peer from the routing table
                if !routing_table.is_empty() {
                    let keys: Vec<&u32> = routing_table.keys().collect();
                    
                    // Directly select random index using our rng::RoutingRng so we don't need rand::thread_rng()
                    let mut index_buf = [0u8; 4];
                    let _ = rng.fill_bytes(&mut index_buf);
                    let random_val = u32::from_be_bytes(index_buf);
                    let random_idx = (random_val as usize) % keys.len();

                    let peer_id = keys[random_idx];
                    if let Some(addr_str) = routing_table.get(peer_id) {
                        let bytes = serialize_packet(&dummy_packet);
                        let _ = courier_send.send_raw(&bytes, addr_str);
                    }
                }
            }
        }
    });

    // Keep daemon running synchronously
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}
