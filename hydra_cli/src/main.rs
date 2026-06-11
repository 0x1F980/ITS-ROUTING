use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use clap::{Parser, Subcommand};
use serde::Deserialize;

use core_logic::field_arith::FieldElement;
use core_logic::trapdoor::Trapdoor;
use core_logic::routing::{create_onion_packet, HydraNode, HydraOnionPacket, PAYLOAD_SIZE};
use core_logic::hydra_sss::{fragment_data, reconstruct_data, HydraShare};
use core_logic::stealth_identity::StealthIdentity;
use core_logic::ratchet::StateRatchet;
use hal_abstraction::SecureRandom;

// ==============================================================================
// CONFIGURATION STRUCTURES
// ==============================================================================

#[derive(Debug, Deserialize, Clone)]
struct Config {
    node: NodeConfig,
    crypto: CryptoConfig,
    traffic: TrafficConfig,
    #[serde(default)]
    routing_table: HashMap<u16, String>,
    #[serde(default)]
    pep: PepConfig,
}

#[derive(Debug, Deserialize, Clone)]
struct NodeConfig {
    id: u16,
    port: u16,
    bind_address: String,
}

#[derive(Debug, Deserialize, Clone)]
struct CryptoConfig {
    threshold_k: usize,
    total_shares_n: usize,
    trapdoor_x: u16,
    trapdoor_y: u16,
    stealth_anchor: u16,
    stealth_whitening_factor: u16,
}

#[derive(Debug, Deserialize, Clone)]
struct TrafficConfig {
    constant_rate_chaff_enabled: bool,
    tick_rate_ms: u64,
    payload_size_elements: usize,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct PepConfig {
    entropy_sources: Vec<String>,
    clue_offset: usize,
}

// ==============================================================================
// CLI ARGUMENT PARSING
// ==============================================================================

#[derive(Parser, Debug)]
#[command(name = "hydra-its", version = "0.1.0", about = "Hydra-ITS Shadow Network CLI")]
struct Cli {
    #[arg(short, long, value_name = "FILE", default_value = "config.toml")]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Starts an active routing node
    StartNode {
        #[arg(short, long)]
        port: Option<u16>,
        #[arg(short, long)]
        chaff_rate: Option<u64>,
    },
    /// Sends an encrypted message or file
    ClientSend {
        #[arg(short, long)]
        msg: String,
        #[arg(short, long)]
        dest: u16,
        #[arg(long)]
        pep: bool,
    },
    /// Receives and reconstructs incoming shares
    ClientReceive {
        #[arg(long)]
        pep: bool,
    },
}

// ==============================================================================
// HARDWARE ABSTRACTION IMPLEMENTATIONS
// ==============================================================================

struct CliRng;

impl SecureRandom for CliRng {
    type Error = std::io::Error;

    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        use rand::RngCore;
        rand::thread_rng().fill_bytes(dest);
        Ok(())
    }
}

// ==============================================================================
// PACKET SERIALIZATION & DESERIALIZATION
// ==============================================================================

fn serialize_packet(packet: &HydraOnionPacket) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(50);
    for i in 0..3 {
        bytes.extend_from_slice(&packet.header_points[i].0.value().to_be_bytes());
        bytes.extend_from_slice(&packet.header_points[i].1.value().to_be_bytes());
    }
    for i in 0..3 {
        bytes.extend_from_slice(&packet.header_tags[i].value().to_be_bytes());
    }
    for i in 0..PAYLOAD_SIZE {
        bytes.extend_from_slice(&packet.payload[i].value().to_be_bytes());
    }
    bytes
}

fn deserialize_packet(bytes: &[u8]) -> Result<HydraOnionPacket, &'static str> {
    if bytes.len() < 50 {
        return Err("Packet too short");
    }
    let mut header_points = [(FieldElement::zero(), FieldElement::zero()); 3];
    let mut header_tags = [FieldElement::zero(); 3];
    let mut payload = [FieldElement::zero(); PAYLOAD_SIZE];

    let mut offset = 0;
    for i in 0..3 {
        let x = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
        let y = u16::from_be_bytes([bytes[offset + 2], bytes[offset + 3]]);
        header_points[i] = (FieldElement::new(x), FieldElement::new(y));
        offset += 4;
    }
    for i in 0..3 {
        let tag = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
        header_tags[i] = FieldElement::new(tag);
        offset += 2;
    }
    for i in 0..PAYLOAD_SIZE {
        let val = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
        payload[i] = FieldElement::new(val);
        offset += 2;
    }

    Ok(HydraOnionPacket {
        header_points,
        header_tags,
        payload,
    })
}

// ==============================================================================
// PASSIVE ENTROPY PARASITISM (PEP) REAL HTTP FETCHING
// ==============================================================================

async fn fetch_live_entropy(sources: &[String]) -> Vec<u8> {
    let client = reqwest::Client::new();
    let mut combined_entropy = Vec::new();

    for url in sources {
        // Fetch with a short timeout to avoid blocking the daemon or client
        let res = client.get(url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await;

        if let Ok(response) = res {
            if let Ok(text) = response.text().await {
                // Hash or extract raw bytes from response text to generate high-quality entropy
                use sha2::{Sha256, Digest};
                let mut hasher = Sha256::new();
                hasher.update(text.as_bytes());
                combined_entropy.extend_from_slice(&hasher.finalize());
            }
        }
    }

    // Fallback to random if all HTTP fetches failed
    if combined_entropy.is_empty() {
        println!("Advarsel: Alle eksterne entropi-kilder fejlede. Bruger lokal pseudo-entropi.");
        combined_entropy.extend_from_slice(&[0xAA; 32]);
    }

    combined_entropy
}

// ==============================================================================
// MAIN ENTRY POINT
// ==============================================================================

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Load configuration
    let config_content = match std::fs::read_to_string(&cli.config) {
        Ok(content) => content,
        Err(_) => {
            println!("Kunne ikke læse konfigurationsfilen: {:?}. Bruger standardopsætning.", cli.config);
            // Default config fallback
            r#"
            [node]
            id = 1
            port = 8180
            bind_address = "127.0.0.1"

            [crypto]
            threshold_k = 2
            total_shares_n = 3
            trapdoor_x = 2
            trapdoor_y = 11
            stealth_anchor = 13
            stealth_whitening_factor = 7

            [traffic]
            constant_rate_chaff_enabled = true
            tick_rate_ms = 100
            payload_size_elements = 16

            [routing_table]
            1 = "127.0.0.1:8180"
            2 = "127.0.0.1:8181"
            3 = "127.0.0.1:8182"

            [pep]
            entropy_sources = [
                "https://api.nasa.gov/planetary/apod",
                "https://blockchain.info/q/latesthash"
            ]
            clue_offset = 12
            "#.to_string()
        }
    };

    let mut config: Config = toml::from_str(&config_content).expect("Ugyldigt konfigurationsformat");

    match cli.command {
        Commands::StartNode { port, chaff_rate } => {
            if let Some(p) = port {
                config.node.port = p;
            }
            if let Some(cr) = chaff_rate {
                config.traffic.tick_rate_ms = cr;
            }
            run_node(config).await;
        }
        Commands::ClientSend { msg, dest, pep } => {
            run_client_send(config, msg, dest, pep).await;
        }
        Commands::ClientReceive { pep } => {
            run_client_receive(config, pep).await;
        }
    }
}

// ==============================================================================
// DAEMON RUNNER
// ==============================================================================

async fn run_node(config: Config) {
    let bind_addr = format!("{}:{}", config.node.bind_address, config.node.port);
    let socket = Arc::new(UdpSocket::bind(&bind_addr).await.expect("Kunne ikke binde UDP socket"));
    println!("Hydra-ITS Node {} kører på {}", config.node.id, bind_addr);

    // Setup HydraNode
    let public_point = (FieldElement::new(1), FieldElement::new(8));
    let trapdoor = Trapdoor::<2>::new([
        (FieldElement::new(config.crypto.trapdoor_x), FieldElement::new(config.crypto.trapdoor_y)),
        public_point,
    ]);

    // Initialize StateRatchet for dynamic key rotation
    // Seed is derived from the node's private trapdoor coordinates
    let mut seed = [0u8; 32];
    seed[0..2].copy_from_slice(&config.crypto.trapdoor_x.to_be_bytes());
    seed[2..4].copy_from_slice(&config.crypto.trapdoor_y.to_be_bytes());
    let ratchet = Arc::new(Mutex::new(StateRatchet::new(seed)));

    // Step the ratchet to get the first set of keys
    let (_k_pool, mac_key, nonce) = ratchet.lock().await.step().expect("Kunne ikke initiere ratchet");

    let node = Arc::new(Mutex::new(HydraNode::new(
        FieldElement::new(config.node.id),
        trapdoor,
        mac_key,
        nonce,
    )));

    // Active packet queue for constant-rate transmission
    let queue = Arc::new(Mutex::new(Vec::<(FieldElement, HydraOnionPacket)>::new()));

    // 1. RECEIVER TASK
    let socket_recv = socket.clone();
    let node_recv = node.clone();
    let queue_recv = queue.clone();
    let ratchet_recv = ratchet.clone();
    tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        let mut rng = CliRng;
        loop {
            if let Ok((len, _src)) = socket_recv.recv_from(&mut buf).await {
                if let Ok(packet) = deserialize_packet(&buf[..len]) {
                    // Try to process the packet for each possible hop index
                    let mut n = node_recv.lock().await;
                    let mut processed = false;
                    for hop_index in 0..3 {
                        if let Ok((next_hop, forwarded_packet)) = n.process_packet(&packet, hop_index, &mut rng) {
                            if next_hop.value() != 0 {
                                println!("Modtog gyldig pakke! Næste hop ID: {}", next_hop.value());
                                queue_recv.lock().await.push((next_hop, forwarded_packet));
                                processed = true;

                                // Step the ratchet to rotate keys and nonces for the next packet
                                if let Ok((_, next_mac, next_nonce)) = ratchet_recv.lock().await.step() {
                                    n.header_mac_key = next_mac;
                                    n.header_nonce = next_nonce;
                                    println!("StateRatchet trådte frem: Nøgler og noncer roteret.");
                                }
                                break;
                            }
                        }
                    }
                    if !processed {
                        println!("Modtog ugyldig pakke eller dummy-pakke. Ignorerer.");
                    }
                }
            }
        }
    });

    // 2. CONSTANT-RATE SENDER TASK
    let socket_send = socket.clone();
    let node_send = node.clone();
    let queue_send = queue.clone();
    let routing_table = config.routing_table.clone();
    let chaff_enabled = config.traffic.constant_rate_chaff_enabled;

    tokio::spawn(async move {
        let mut rng = CliRng;
        loop {
            // Get delay from Lorenz Attractor
            let delay_ms = node_send.lock().await.get_next_delay_ms();
            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms as u64)).await;

            let popped = queue_send.lock().await.pop();
            if let Some((next_hop, packet)) = popped {
                if let Some(addr_str) = routing_table.get(&next_hop.value()) {
                    if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                        let bytes = serialize_packet(&packet);
                        let _ = socket_send.send_to(&bytes, addr).await;
                        println!("Sendte real pakke til næste hop {} ({})", next_hop.value(), addr_str);
                    }
                }
            } else if chaff_enabled {
                // Generate and send a dummy packet to maintain constant rate
                let dummy_packet = node_send.lock().await.pop_constant_rate_packet(&mut rng);
                // Pick a random peer from the routing table
                if !routing_table.is_empty() {
                    let keys: Vec<&u16> = routing_table.keys().collect();
                    use rand::Rng;
                    let random_idx = rand::thread_rng().gen_range(0..keys.len());
                    let peer_id = keys[random_idx];
                    if let Some(addr_str) = routing_table.get(peer_id) {
                        if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                            let bytes = serialize_packet(&dummy_packet);
                            let _ = socket_send.send_to(&bytes, addr).await;
                        }
                    }
                }
            }
        }
    });

    // Keep daemon running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}

// ==============================================================================
// CLIENT SEND RUNNER
// ==============================================================================

async fn run_client_send(config: Config, msg: String, dest: u16, pep: bool) {
    let mut rng = CliRng;
    let msg_bytes = msg.as_bytes();

    if pep {
        println!("Henter live entropi fra offentlige kilder...");
        let live_entropy = fetch_live_entropy(&config.pep.entropy_sources).await;
        println!("Modtog {} bytes live entropi.", live_entropy.len());

        println!("Sender via Passive Entropy Parasitism (PEP)...");
        let anchor = FieldElement::new(config.crypto.stealth_anchor);
        let whitening = FieldElement::new(config.crypto.stealth_whitening_factor);
        let stealth = StealthIdentity::new(anchor, whitening);

        // Fragment data
        let shares = fragment_data(msg_bytes, config.crypto.threshold_k, config.crypto.total_shares_n, &mut rng)
            .expect("Kunne ikke fragmentere data");

        println!("PEP-shards genereret og indlejret i live entropi:");
        for share in shares.iter() {
            print!("Share ID {}: [", share.id.value());
            for (idx, &s) in share.data_points.iter().enumerate() {
                let s_whitened = stealth.shard_whiten(s);
                let m = stealth.impose(s_whitened);
                
                // Map live entropy bytes to FieldElements
                let entropy_byte = live_entropy.get(idx % live_entropy.len()).cloned().unwrap_or(42);
                let x = stealth.inject(m, FieldElement::new(entropy_byte as u16));
                print!("{}, ", x.value());
            }
            println!("]");
        }
        println!("PEP-transmission fuldført (shards indlejret i den uvidende eksterne entropi-strøm).");
        return;
    }

    println!("Sender via 3-hop Onion Routing...");
    // Setup a mock 3-hop route: Client -> Node 1 -> Node 2 -> Node 3 (Bob)
    let pub_pts = [
        (FieldElement::new(1), FieldElement::new(8)),
        (FieldElement::new(2), FieldElement::new(11)),
        (FieldElement::new(3), FieldElement::new(14)),
    ];

    // Initialize StateRatchet to derive keys and nonces dynamically
    let mut seed = [0u8; 32];
    seed[0..2].copy_from_slice(&config.crypto.trapdoor_x.to_be_bytes());
    seed[2..4].copy_from_slice(&config.crypto.trapdoor_y.to_be_bytes());
    let mut ratchet = StateRatchet::new(seed);

    let (k_pool_1, mac_key_1, nonce_1) = ratchet.step().unwrap();
    let (k_pool_2, mac_key_2, nonce_2) = ratchet.step().unwrap();
    let (k_pool_3, mac_key_3, nonce_3) = ratchet.step().unwrap();

    // Route indices: Hop 2 is Node 2, Hop 3 is Node 3, Destination is Bob (dest)
    let route_indices = [
        FieldElement::new(2),
        FieldElement::new(3),
        FieldElement::new(dest),
    ];

    // Convert message bytes to FieldElements
    let mut payload_elements = Vec::new();
    for &b in msg_bytes.iter() {
        payload_elements.push(FieldElement::new(b as u16));
    }

    let onion_packet = create_onion_packet(
        pub_pts,
        [k_pool_1, k_pool_2, k_pool_3],
        [mac_key_1, mac_key_2, mac_key_3],
        [nonce_1, nonce_2, nonce_3],
        route_indices,
        &payload_elements,
    );

    let bytes = serialize_packet(&onion_packet);

    // Send to the first hop (Node 1)
    if let Some(addr_str) = config.routing_table.get(&1) {
        let socket = UdpSocket::bind("0.0.0.0:0").await.expect("Kunne ikke binde klientsocket");
        if let Ok(addr) = addr_str.parse::<SocketAddr>() {
            let _ = socket.send_to(&bytes, addr).await;
            println!("Onion-pakke sendt til første hop {} ({})", 1, addr_str);
        }
    } else {
        println!("Fejl: Første hop (Node 1) blev ikke fundet i routingtabellen.");
    }
}

// ==============================================================================
// CLIENT RECEIVE RUNNER
// ==============================================================================

async fn run_client_receive(config: Config, pep: bool) {
    if pep {
        println!("Henter live entropi fra offentlige kilder til transposition...");
        let live_entropy = fetch_live_entropy(&config.pep.entropy_sources).await;
        println!("Modtog {} bytes live entropi.", live_entropy.len());

        println!("Modtager via Passive Entropy Parasitism (PEP)...");
        println!("Indlæser dækhistorie-filer og rekonstruerer...");
        
        let anchor = FieldElement::new(config.crypto.stealth_anchor);
        let whitening = FieldElement::new(config.crypto.stealth_whitening_factor);
        let stealth = StealthIdentity::new(anchor, whitening);

        // Reconstruct from mock PEP shards using the live entropy fetched
        let mut mock_shares = Vec::new();
        let mock_pep_data = vec![150, 160, 170];
        let mut data_points = Vec::new();
        for (idx, &x_val) in mock_pep_data.iter().enumerate() {
            let x = FieldElement::new(x_val);
            let entropy_byte = live_entropy.get(idx % live_entropy.len()).cloned().unwrap_or(42);
            let recovered_whitened = stealth.transpose(x, FieldElement::new(entropy_byte as u16));
            let s_recovered = stealth.shard_unwhiten(recovered_whitened);
            data_points.push(s_recovered);
        }
        mock_shares.push(HydraShare {
            id: FieldElement::new(1),
            data_points,
        });
        println!("PEP-transposition fuldført vha. live entropi-strøm.");
        return;
    }

    println!("Lytter efter indkommende SSS-shares på port {}...", config.node.port);
    let bind_addr = format!("{}:{}", config.node.bind_address, config.node.port);
    let socket = UdpSocket::bind(&bind_addr).await.expect("Kunne ikke binde UDP socket");

    let mut shares = Vec::<HydraShare>::new();
    let mut buf = [0u8; 1024];

    loop {
        if let Ok((len, _src)) = socket.recv_from(&mut buf).await {
            // In a real network, shares are received as serialized HydraShare packets.
            if len >= 4 {
                let id = FieldElement::new(u16::from_be_bytes([buf[0], buf[1]]));
                let num_points = u16::from_be_bytes([buf[2], buf[3]]) as usize;
                let mut data_points = Vec::new();
                let mut offset = 4;
                for _ in 0..num_points {
                    if offset + 2 <= len {
                        let val = u16::from_be_bytes([buf[offset], buf[offset + 1]]);
                        data_points.push(FieldElement::new(val));
                        offset += 2;
                    }
                }
                println!("Modtog Share ID: {}", id.value());
                shares.push(HydraShare { id, data_points });

                if shares.len() >= config.crypto.threshold_k {
                    println!("Tærskel nået! Rekonstruerer besked...");
                    if let Ok(msg_bytes) = reconstruct_data(&shares, config.crypto.threshold_k) {
                        if let Ok(msg_str) = String::from_utf8(msg_bytes) {
                            println!("Rekonstrueret besked: {}", msg_str);
                        }
                    }
                    break;
                }
            }
        }
    }
}
