use std::net::{SocketAddr, UdpSocket};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use its_transport::field_arith::FieldElement;
#[cfg(feature = "dev-onion-mix")]
use its_transport::onion::create_onion_packet;
use its_transport::{fragment_data, reconstruct_data, EpochCellState, SssPackedShare};
use its_transport::TransportOtpRatchet;
use its_transport::SecureRandom;

use crate::config::Config;
use crate::courier::{build_epoch_courier_from, EpochCourierBuild, ZeroizedBuffer};
use crate::availability_ledger;
use crate::valid_forward_party::{establish_canonical, record_mirror_mismatch, ValidForwardState};
use crate::pool_mailbox::PoolMailbox;
#[cfg(feature = "pool")]
use crate::cover_transport::{EpochLoop, PoolPlusCoverHarvest};
#[cfg(feature = "dev-onion-mix")]
use crate::packet::{deserialize_packet, deserialize_share, payload_to_bytes, serialize_packet, serialize_share};
use crate::ratchet::resolve_pool_ratchet_seed;
use crate::rng;
use crate::ridges;

#[cfg(feature = "dev-direct-udp")]
fn send_fragmented_udp(
    config: &Config,
    dest: u32,
    msg_bytes: &[u8],
    rng: &mut rng::RoutingRng,
) -> Result<(), ()> {
    let shares = fragment_data(
        msg_bytes,
        config.crypto.threshold_k,
        config.crypto.total_shares_n,
        rng,
    )
    .map_err(|_| ())?;
    let dest_addr = config.routing_table.get(&dest).ok_or(())?;
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|_| ())?;
    let addr: SocketAddr = dest_addr.parse().map_err(|_| ())?;
    for share in shares {
        let bytes = serialize_share(&share);
        socket.send_to(&bytes, addr).map_err(|_| ())?;
    }
    Ok(())
}

#[cfg(feature = "dev-onion-mix")]
fn forward_onion_through_mesh(config: &Config, packet: &its_transport::onion::MorphicOnionPacket) -> Result<(), ()> {
    let bytes = serialize_packet(packet);
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|_| ())?;
    let hop1 = config.routing_table.get(&1).ok_or(())?;
    let addr: SocketAddr = hop1.parse().map_err(|_| ())?;
    socket.send_to(&bytes, addr).map_err(|_| ())?;
    Ok(())
}

#[cfg(feature = "pool")]
fn run_pool_send(
    config: &Config,
    msg_bytes: &[u8],
    ratchet_seed_file: &PathBuf,
) {
    let Some(seed) = resolve_pool_ratchet_seed(ratchet_seed_file) else {
        return;
    };
    let sss_k = if config.pool.sss_k > 0 {
        config.pool.sss_k
    } else {
        config.crypto.threshold_k
    };
    let sss_n = if config.pool.sss_n > 0 {
        config.pool.sss_n
    } else {
        config.crypto.total_shares_n
    };
    let mut cell_state = match EpochCellState::new(seed, config.pool.cell_size_l, sss_k, sss_n) {
        Ok(s) => s,
        Err(()) => {
            println!("Error: invalid pool cell parameters.");
            return;
        }
    };
    let mut rng = rng::RoutingRng;
    if cell_state.queue_sss_payload(msg_bytes, &mut rng).is_err() {
        println!("Error: could not queue SSS payload for pool transmission.");
        return;
    }
    let payload_epochs = cell_state.queued_epochs();
    let chaff_epochs = cell_state.fountain_extra_chaff_epochs(config.pool.fountain_enabled);
    let total_epochs = payload_epochs + chaff_epochs;
    let actor = config.node.id;
    if !availability_ledger::pool_publish_allowed(actor) {
        println!(
            "Error: send rights revoked for node {actor} ({} strikes ≥ threshold). \
             Availability ledger forbids pool publish.",
            availability_ledger::strike_count(actor)
        );
        return;
    }
    let valid_fwd = Arc::new(Mutex::new(ValidForwardState::new()));
    let courier = build_epoch_courier_from(EpochCourierBuild {
        pool_file: &config.pool.pool_file,
        pool_url: &config.pool.pool_url,
        multi_pool_urls: &config.pool.multi_pool_urls,
        witness_pool_urls: &config.pool.witness_pool_urls,
        consensus_k: config.pool.consensus_k,
        valid_fwd_window: config.pool.valid_fwd_window,
        valid_fwd_state: Arc::clone(&valid_fwd),
    });
    println!(
        "UES Pool send (L3): {} payload epochs + {chaff_epochs} chaff, cell_size_L={}, pool={}",
        payload_epochs,
        config.pool.cell_size_l,
        config.pool.pool_file
    );
    let interval = Duration::from_millis(config.pool.epoch_interval_ms.max(1));
    for epoch in 0..total_epochs {
        let (_, cell) = match cell_state.step(&mut rng) {
            Ok(v) => v,
            Err(()) => {
                println!("Error: epoch cell step failed at epoch {epoch}.");
                return;
            }
        };
        if courier.publish_cell(epoch as u64, &cell).is_err() {
            println!("Error: failed to publish cell at epoch {epoch}.");
            return;
        }
        if epoch + 1 < total_epochs {
            thread::sleep(interval);
        }
    }
    println!(
        "Published {total_epochs} epoch cells ({} bytes payload) to UES Monocell Pool.",
        msg_bytes.len()
    );
}

#[cfg(feature = "pool")]
fn run_pool_receive(
    config: &Config,
    ratchet_seed_file: &PathBuf,
    out_path: Option<&PathBuf>,
    timeout_secs: u64,
    continuous: bool,
    from_epoch: u64,
    mailbox: Option<PoolMailbox>,
) {
    let Some(seed) = resolve_pool_ratchet_seed(ratchet_seed_file) else {
        return;
    };
    let sss_k = if config.pool.sss_k > 0 {
        config.pool.sss_k
    } else {
        config.crypto.threshold_k
    };
    let sss_n = if config.pool.sss_n > 0 {
        config.pool.sss_n
    } else {
        config.crypto.total_shares_n
    };
    let mut cell_state = match EpochCellState::new(seed, config.pool.cell_size_l, sss_k, sss_n) {
        Ok(s) => s,
        Err(()) => {
            println!("Error: invalid pool cell parameters.");
            return;
        }
    };
    let valid_fwd = Arc::new(Mutex::new(ValidForwardState::new()));
    let cover = PoolPlusCoverHarvest::new(
        &config.pool.pool_file,
        &config.pool.pool_url,
        &config.pool.multi_pool_urls,
        &config.pool.witness_pool_urls,
        config.pool.consensus_k,
        config.pool.valid_fwd_window,
        &config.aeh.entropy_sources,
        Arc::clone(&valid_fwd),
    );
    println!(
        "UES Pool receive (L3'): cover+pool harvest every {}ms, threshold k={}, cover_sources={}",
        config.pool.epoch_interval_ms,
        sss_k,
        config.aeh.entropy_sources.len()
    );
    let ticker = EpochLoop::new(config.pool.epoch_interval_ms);
    let mut shares: Vec<SssPackedShare> = Vec::new();
    let mut next_epoch = from_epoch;
    let max_shares = cell_state.fountain_max_shares(config.pool.fountain_enabled);
    let mb = mailbox.unwrap_or_default();
    if mb.strict && mb.namespace != 0 {
        println!(
            "PoolMailbox: strict receive namespace=0x{:08X} (hint in ciphertext only).",
            mb.namespace
        );
    }

    loop {
        let bundle = match cover.harvest_epoch(next_epoch) {
            Ok(b) => b,
            Err(e) => {
                println!("Error harvesting pool/cover epoch: {e}");
                return;
            }
        };
        if bundle.cover_bytes > 0 {
            println!("Cover harvest: {} bytes from E-sources.", bundle.cover_bytes);
        }
        let cells_empty = bundle.pool_cells.is_empty();
        for (epoch, cell) in bundle.pool_cells {
            match cell_state.verify_cell(epoch, &cell) {
                Ok(Some(share)) => {
                    if let Ok(mut st) = valid_fwd.lock() {
                        establish_canonical(&mut st, epoch, &cell);
                    }
                    let sid = share.id.value();
                    if mb.accept_verified_share(sid) && !shares.iter().any(|s| s.id == share.id) {
                        shares.push(share);
                        println!("Verified share ID {sid} at epoch {epoch}.");
                    }
                }
                Ok(None) => {}
                Err(()) => {
                    if let Ok(mut st) = valid_fwd.lock() {
                        if st.canonical.get(epoch).is_some() {
                            record_mirror_mismatch(&mut st, "pool-harvest", epoch);
                        }
                    }
                    println!("Warning: cell verify failed at epoch {epoch}.");
                }
            }
            next_epoch = epoch.saturating_add(1);
        }

        if shares.len() >= sss_k {
            match reconstruct_data(&shares, sss_k) {
                Ok(msg_bytes) => {
                    if !mb.accept_reconstructed_payload(&msg_bytes) {
                        println!(
                            "PoolMailbox: reconstructed {} bytes — wire/OTM gate rejected (scanning).",
                            msg_bytes.len()
                        );
                        if shares.len() >= max_shares {
                            shares.clear();
                        }
                    } else {
                        if let Some(out) = out_path {
                            if std::fs::write(out, &msg_bytes).is_err() {
                                println!("Error: could not write output file.");
                                return;
                            }
                            println!("Wrote {} bytes -> {:?}", msg_bytes.len(), out);
                        } else {
                            let secured = ZeroizedBuffer::new(msg_bytes);
                            if let Ok(msg_str) = String::from_utf8(secured.data.clone()) {
                                println!("Reconstructed pool message: {msg_str}");
                            }
                        }
                        return;
                    }
                }
                Err(()) => {
                    if !config.pool.fountain_enabled || shares.len() >= max_shares {
                        println!(
                            "Error: have {} verified shares but reconstruction failed (need k={}).",
                            shares.len(),
                            sss_k
                        );
                        return;
                    }
                }
            }
        }

        if ticker.timed_out(timeout_secs) {
            println!(
                "Receive timeout after {timeout_secs}s (have {} shares, need {sss_k}).",
                shares.len()
            );
            break;
        }
        if !continuous && cells_empty && shares.is_empty() {
            println!("No pool cells found (single pass, use --continuous to poll).");
            break;
        }
        if !continuous && !cells_empty && shares.len() >= sss_k {
            break;
        }

        ticker.wait_tick();
    }
    println!(
        "Error: could not reconstruct from pool (have {} shares, need {}).",
        shares.len(),
        sss_k
    );
}

#[cfg(feature = "aeh")]
fn run_aeh_send(
    config: &Config,
    msg_bytes: &[u8],
    ratchet_seed_file: &PathBuf,
) {
    use crate::aeh_carrier::{production_channels, AehCarrier, FileAehCarrier};

    let seed = match resolve_pool_ratchet_seed(ratchet_seed_file) {
        Some(s) => s,
        None => return,
    };
    let sss_k = if config.pool.sss_k > 0 {
        config.pool.sss_k
    } else {
        config.crypto.threshold_k
    };
    let sss_n = if config.pool.sss_n > 0 {
        config.pool.sss_n
    } else {
        config.crypto.total_shares_n
    };
    let mut cell_state = match EpochCellState::new(seed, config.pool.cell_size_l, sss_k, sss_n) {
        Ok(s) => s,
        Err(()) => {
            println!("Error: invalid AEH cell parameters.");
            return;
        }
    };
    let mut rng = rng::RoutingRng;
    if cell_state.queue_sss_payload(msg_bytes, &mut rng).is_err() {
        println!("Error: could not queue SSS payload for AEH transmission.");
        return;
    }
    let aeh_dir = format!("{}_aeh", config.pool.pool_file);
    let carrier = FileAehCarrier::new(&aeh_dir);
    let channels = production_channels();
    let payload_epochs = cell_state.queued_epochs();
    let total_epochs = payload_epochs + 2;
    println!(
        "AEH last-resort send (L3): {} payload epochs + 2 chaff, φ ~ D_benign, dir={aeh_dir}",
        payload_epochs
    );
    for epoch in 0..total_epochs {
        let (_, cell) = match cell_state.step(&mut rng) {
            Ok(v) => v,
            Err(()) => {
                println!("Error: AEH epoch cell step failed at epoch {epoch}.");
                return;
            }
        };
        let channel = channels[epoch as usize % channels.len()];
        if carrier.publish(epoch as u64, &cell, channel).is_err() {
            println!("Error: failed to publish AEH observation at epoch {epoch}.");
            return;
        }
    }
    println!(
        "Published {total_epochs} AEH observations ({} bytes payload).",
        msg_bytes.len()
    );
}

#[cfg(feature = "aeh")]
fn run_aeh_receive(
    config: &Config,
    ratchet_seed_file: &PathBuf,
    out_path: Option<&PathBuf>,
) {
    use crate::aeh_carrier::{AehCarrier, FileAehCarrier};

    let seed = match resolve_pool_ratchet_seed(ratchet_seed_file) {
        Some(s) => s,
        None => return,
    };
    let sss_k = if config.pool.sss_k > 0 {
        config.pool.sss_k
    } else {
        config.crypto.threshold_k
    };
    let sss_n = if config.pool.sss_n > 0 {
        config.pool.sss_n
    } else {
        config.crypto.total_shares_n
    };
    let mut cell_state = match EpochCellState::new(seed, config.pool.cell_size_l, sss_k, sss_n) {
        Ok(s) => s,
        Err(()) => {
            println!("Error: invalid AEH cell parameters.");
            return;
        }
    };
    let aeh_dir = format!("{}_aeh", config.pool.pool_file);
    let carrier = FileAehCarrier::new(&aeh_dir);
    println!(
        "AEH receive (L3'): constant harvest from benign E-channels, threshold k={sss_k}"
    );
    let observations = match carrier.harvest_all() {
        Ok(o) => o,
        Err(e) => {
            println!("Error harvesting AEH observations: {e}");
            return;
        }
    };
    let mut shares: Vec<SssPackedShare> = Vec::new();
    for (epoch, text, channel) in observations {
        let cell = match channel.extract_cell(&text) {
            Some(c) => c,
            None => continue,
        };
        match cell_state.verify_cell(epoch, &cell) {
            Ok(Some(share)) => {
                let sid = share.id.value();
                if !shares.iter().any(|s| s.id == share.id) {
                    shares.push(share);
                    println!("Verified AEH share ID {sid} at epoch {epoch} via {}.", channel.name());
                }
            }
            Ok(None) => {}
            Err(()) => {
                println!("Warning: AEH cell verify failed at epoch {epoch}.");
            }
        }
    }
    if shares.len() >= sss_k {
        match reconstruct_data(&shares, sss_k) {
            Ok(msg_bytes) => {
                if let Some(out) = out_path {
                    if std::fs::write(out, &msg_bytes).is_err() {
                        println!("Error: could not write output file.");
                        return;
                    }
                    println!("Wrote {} bytes -> {:?}", msg_bytes.len(), out);
                } else {
                    let secured = ZeroizedBuffer::new(msg_bytes);
                    if let Ok(msg_str) = String::from_utf8(secured.data.clone()) {
                        println!("Reconstructed AEH message: {msg_str}");
                    }
                }
                return;
            }
            Err(()) => {}
        }
    }
    println!(
        "Error: could not reconstruct from AEH (have {} shares, need {}).",
        shares.len(),
        sss_k
    );
}

pub fn run_client_send(
    config: Config,
    msg: String,
    file_path: PathBuf,
    dest: u32,
    aeh: bool,
    continuous: bool,
    pool: bool,
    ratchet_seed_file: PathBuf,
    fe: ridges::fingerprint_erasure::FingerprintErasureSendOptions,
) {
    let mut rng = rng::RoutingRng;

    let raw_payload = if !file_path.as_os_str().is_empty() {
        match std::fs::read(&file_path) {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("Error: Could not read file: {:?}", e);
                return;
            }
        }
    } else if msg.is_empty() {
        println!("Error: client-send requires --msg or --file.");
        return;
    } else {
        msg.into_bytes()
    };

    #[cfg(feature = "fingerprint-erasure")]
    let payload = match ridges::fingerprint_erasure::prepare_send_payload(&raw_payload, &fe) {
        Ok(p) => p,
        Err(e) => {
            println!("Error during fingerprint-erasure: {e}");
            return;
        }
    };
    #[cfg(not(feature = "fingerprint-erasure"))]
    let payload = ridges::fingerprint_erasure::prepare_send_payload(&raw_payload, &fe)
        .expect("fingerprint-erasure disabled");

    #[cfg(feature = "fingerprint-erasure")]
    if fe.enabled {
        if fe.otp_wire {
            println!(
                "Fingerprint-erasure v2: Γ(max) + OTP wire ({} bytes sent).",
                payload.len()
            );
        } else {
            println!(
                "Fingerprint-erasure v2: Γ({}) normalized ({} bytes sent).",
                if fe.mode == its_fingerprint_erasure::ErasureMode::Standard {
                    "standard"
                } else if fe.mode == its_fingerprint_erasure::ErasureMode::Extended {
                    "extended"
                } else {
                    "minimal"
                },
                payload.len()
            );
        }
    }

    let msg_bytes = payload.as_slice();

    let use_pool = pool || config.pool.transport_mode == "pool";
    #[cfg(feature = "pool")]
    if use_pool && !aeh {
        run_pool_send(&config, msg_bytes, &ratchet_seed_file);
        return;
    }
    #[cfg(not(feature = "pool"))]
    if use_pool {
        println!("Error: pool transport requires `pool` feature.");
        return;
    }

    #[cfg(feature = "aeh")]
    if aeh {
        run_aeh_send(&config, msg_bytes, &ratchet_seed_file);
        return;
    }

    #[cfg(feature = "dev-direct-udp")]
    if !aeh && (!file_path.as_os_str().is_empty() || msg_bytes.len() > 12) {
        match send_fragmented_udp(&config, dest, msg_bytes, &mut rng) {
            Ok(()) => {
                println!(
                    "Sent {} bytes as {} SSS fragments to destination {} (ITS transport).",
                    msg_bytes.len(),
                    config.crypto.total_shares_n,
                    dest
                );
                return;
            }
            Err(()) => {
                println!("Error: fragmented UDP send failed (check routing_table and dest).");
                return;
            }
        }
    }

    #[cfg(feature = "dev-onion-mix")]
    {
    println!("Sender via 3-hop Onion Routing...");
    // Setup a mock 3-hop route: Client -> Node 1 -> Node 2 -> Node 3 (Bob)
    let pub_pts = [
        (FieldElement::new(1), FieldElement::new(8)),
        (FieldElement::new(2), FieldElement::new(11)),
        (FieldElement::new(3), FieldElement::new(14)),
    ];

    // Initialize TransportOtpRatchet to derive keys and nonces dynamically
    let mut seed = [0u8; 32];
    seed[0..4].copy_from_slice(&config.crypto.trapdoor_x.to_be_bytes());
    seed[4..8].copy_from_slice(&config.crypto.trapdoor_y.to_be_bytes());
    let mut ratchet = TransportOtpRatchet::new(seed);

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
        payload_elements.push(FieldElement::new(b as u32));
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

    if forward_onion_through_mesh(&config, &onion_packet).is_ok() {
        println!("Onion packet sent to first hop 1");
    } else if let Some(addr_str) = config.routing_table.get(&1) {
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Could not bind client socket");
        if let Ok(addr) = addr_str.parse::<SocketAddr>() {
            let _ = socket.send_to(&bytes, addr);
            println!("Onion packet sent to first hop {} ({})", 1, addr_str);
        }
    } else {
        println!("Error: First hop (Node 1) not found in routing table.");
    }
    }
    #[cfg(not(feature = "dev-onion-mix"))]
    {
        println!("Error: no transport mode selected. Use --pool (default) or build with dev-direct-udp / dev-onion-mix.");
    }
}

// ==============================================================================
// CLIENT RECEIVE RUNNER
// ==============================================================================

pub fn run_client_receive(
    config: Config,
    aeh: bool,
    continuous: bool,
    pool: bool,
    ratchet_seed_file: PathBuf,
    out_path: Option<PathBuf>,
    timeout_secs: u64,
    from_epoch: u64,
    mailbox: Option<PoolMailbox>,
) {
    let use_pool = pool || config.pool.transport_mode == "pool";
    #[cfg(feature = "pool")]
    if use_pool && !aeh {
        run_pool_receive(
            &config,
            &ratchet_seed_file,
            out_path.as_ref(),
            timeout_secs,
            continuous,
            from_epoch,
            mailbox,
        );
        return;
    }
    #[cfg(not(feature = "pool"))]
    if use_pool {
        println!("Error: pool transport requires `pool` feature.");
        return;
    }

    if aeh {
        #[cfg(feature = "aeh")]
        {
            run_aeh_receive(&config, &ratchet_seed_file, out_path.as_ref());
            return;
        }
        #[cfg(not(feature = "aeh"))]
        {
            println!("Error: AEH requires `aeh` feature.");
            return;
        }
    }

    #[cfg(feature = "dev-direct-udp")]
    {
    println!("Listening for incoming SSS shares on port {}...", config.node.port);
    let bind_addr = format!("{}:{}", config.node.bind_address, config.node.port);
    let socket = UdpSocket::bind(&bind_addr).expect("Could not bind UDP socket");
    if timeout_secs > 0 {
        let _ = socket.set_read_timeout(Some(Duration::from_secs(timeout_secs)));
    }

    let mut shares = Vec::<SssPackedShare>::new();
    let mut buf = [0u8; 65536];

    loop {
        match socket.recv_from(&mut buf) {
            Ok((len, _src)) => {
                if let Ok(share) = deserialize_share(&buf[..len]) {
                    if shares.iter().any(|s| s.id == share.id) {
                        println!("Duplicate Share ID: {} (ignored)", share.id.value());
                    } else {
                        println!("Received Share ID: {}", share.id.value());
                        shares.push(share);
                    }
                } else if let Ok(packet) = deserialize_packet(&buf[..len]) {
                    let bytes = payload_to_bytes(&packet.payload);
                    if !bytes.is_empty() {
                        if out_path.is_none() {
                            let secured = ZeroizedBuffer::new(bytes);
                            if let Ok(msg_str) = String::from_utf8(secured.data.clone()) {
                                println!("Onion payload: {}", msg_str);
                            }
                        } else {
                            println!(
                                "Ignoring chaff/onion packet ({} bytes); waiting for SSS threshold.",
                                bytes.len()
                            );
                        }
                    }
                } else if len >= 8 && len < 100 {
                    if let Ok(share) = deserialize_share(&buf[..len]) {
                        println!("Received Share ID: {}", share.id.value());
                        shares.push(share);
                    }
                }

                if shares.len() >= config.crypto.threshold_k {
                    println!("Threshold reached! Reconstructing message...");
                    if let Ok(msg_bytes) = reconstruct_data(&shares, config.crypto.threshold_k) {
                        if let Some(ref out) = out_path {
                            let _ = std::fs::write(out, &msg_bytes);
                            println!("Wrote {} bytes -> {:?}", msg_bytes.len(), out);
                            return;
                        }
                        let secured = ZeroizedBuffer::new(msg_bytes);
                        if let Ok(msg_str) = String::from_utf8(secured.data.clone()) {
                            println!("Reconstructed message: {}", msg_str);
                        }
                    }
                    if out_path.is_some() {
                        return;
                    }
                    break;
                }
            }
            Err(e) => {
                if timeout_secs > 0 {
                    println!("Receive timeout or error: {e}");
                    break;
                }
            }
        }
    }
    }
    #[cfg(not(feature = "dev-direct-udp"))]
    {
        let _ = timeout_secs;
        println!("Error: no receive transport mode. Use --pool (default) or build with dev-direct-udp.");
    }
}
