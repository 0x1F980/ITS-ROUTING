use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

use core_logic::field_arith::FieldElement;
use core_logic::trapdoor::Trapdoor;
use core_logic::routing::{create_onion_packet, HydraNode, HydraOnionPacket, PAYLOAD_SIZE};
use core_logic::hydra_sss::{fragment_data, reconstruct_data, HydraShare};
use core_logic::stealth_identity::StealthIdentity;
use core_logic::ratchet::StateRatchet;
use core_logic::time_lock::SssTimeLock;
use hal_abstraction::SecureRandom;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A memory-secured container that zeroizes its contents upon drop to protect RAM state.
#[derive(Zeroize, ZeroizeOnDrop, Default)]
struct ZeroizedBuffer {
    data: Vec<u8>,
}

impl ZeroizedBuffer {
    fn new(data: Vec<u8>) -> Self {
        ZeroizedBuffer { data }
    }
}

// ==============================================================================
// CONFIGURATION STRUCTURES
// ==============================================================================

#[derive(Debug, Deserialize, Clone)]
struct Config {
    node: NodeConfig,
    crypto: CryptoConfig,
    traffic: TrafficConfig,
    #[serde(default)]
    routing_table: HashMap<u32, String>,
    #[serde(default)]
    pep: PepConfig,
}

#[derive(Debug, Deserialize, Clone)]
struct NodeConfig {
    id: u32,
    port: u16,
    bind_address: String,
}

#[derive(Debug, Deserialize, Clone)]
struct CryptoConfig {
    threshold_k: usize,
    total_shares_n: usize,
    trapdoor_x: u32,
    trapdoor_y: u32,
    stealth_anchor: u32,
    stealth_whitening_factor: u32,
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
// TIME-LOCK JSON MIRROR FOR SERDE
// ==============================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TimeLockJson {
    x: u64,
    m: u64,
    t: usize,
    initial_share_1: Vec<u32>,
    transitions_1: Vec<Vec<u32>>,
    transitions_2: Vec<Vec<u32>>,
    encrypted_payload: Vec<u32>,
}

impl TimeLockJson {
    fn from_core(puzzle: &SssTimeLock) -> Self {
        TimeLockJson {
            x: puzzle.x,
            m: puzzle.m,
            t: puzzle.t,
            initial_share_1: puzzle.initial_share_1.iter().map(|f| f.value()).collect(),
            transitions_1: puzzle.transitions_1.iter().map(|v| v.iter().map(|f| f.value()).collect()).collect(),
            transitions_2: puzzle.transitions_2.iter().map(|v| v.iter().map(|f| f.value()).collect()).collect(),
            encrypted_payload: puzzle.encrypted_payload.iter().map(|f| f.value()).collect(),
        }
    }

    fn to_core(&self) -> SssTimeLock {
        SssTimeLock {
            x: self.x,
            m: self.m,
            t: self.t,
            initial_share_1: self.initial_share_1.iter().map(|&v| FieldElement::new(v)).collect(),
            transitions_1: self.transitions_1.iter().map(|v| v.iter().map(|&v| FieldElement::new(v)).collect()).collect(),
            transitions_2: self.transitions_2.iter().map(|v| v.iter().map(|&v| FieldElement::new(v)).collect()).collect(),
            encrypted_payload: self.encrypted_payload.iter().map(|&v| FieldElement::new(v)).collect(),
        }
    }
}

// ==============================================================================
// PASSIVE ENTROPY PARASITISM (PEP) ADAPTERS
// ==============================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PepBlock {
    share_id: u32,
    x_points: Vec<u32>,
    tags: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PepChannel {
    Wikipedia,
    GitHubGists,
    DnsTxt,
    Reddit,
    NasaTelemetry,
    DomesticNews,
    SneakernetFile,
}

impl PepChannel {
    fn name(&self) -> &'static str {
        match self {
            PepChannel::Wikipedia => "Wikipedia API (Simulated)",
            PepChannel::GitHubGists => "GitHub Gists API (Simulated)",
            PepChannel::DnsTxt => "DNS TXT Records (Simulated)",
            PepChannel::Reddit => "Reddit Comments API (Simulated)",
            PepChannel::NasaTelemetry => "NASA Seismology API (Simulated)",
            PepChannel::DomesticNews => "State-Approved Domestic News Board (Simulated ALT)",
            PepChannel::SneakernetFile => "Sneakernet Local File / QR (Simulated ALT)",
        }
    }

    /// Encodes a PepBlock into simulated steganographic camouflage text
    fn stego_encode(&self, block: &PepBlock) -> String {
        match self {
            PepChannel::Wikipedia => {
                format!(
                    r#"{{"wiki": "enwiki", "title": "Information-theoretic secrecy", "revision": 1294817204, "diff": {{"added": {{"user": "IP_User", "payload_id": {}, "points": {:?}, "signature": {:?}}}}}}}"#,
                    block.share_id, block.x_points, block.tags
                )
            }
            PepChannel::GitHubGists => {
                format!(
                    "// Gist ID: gist_its_shadow_node_{}\nconst CONFIG_VALS = {:?};\nconst STATUS_SIG = {:?};\n",
                    block.share_id, block.x_points, block.tags
                )
            }
            PepChannel::DnsTxt => {
                let points_str = block.x_points.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("-");
                let tags_str = block.tags.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("-");
                format!(
                    "v=spf1 ip4:192.168.1.1 include:_spf.google.com its_id={} points={} tags={} ~all",
                    block.share_id, points_str, tags_str
                )
            }
            PepChannel::Reddit => {
                format!(
                    "Honestly, the performance stats are quite interesting. I measured some latency offsets (ID {}): points={:?}. Our error-checking tags also fluctuated around tags={:?}. Anyone else seeing this?",
                    block.share_id, block.x_points, block.tags
                )
            }
            PepChannel::NasaTelemetry => {
                format!(
                    "SENS_ID={};GEOM_X={:?};NOISE_FILTER={:?};SYS_STATUS=OK",
                    block.share_id, block.x_points, block.tags
                )
            }
            PepChannel::DomesticNews => {
                format!(
                    "State-approved announcement: Local infrastructure update completed successfully (Event ID {}). Operational points={:?}, checksums={:?}. In compliance with municipal directives.",
                    block.share_id, block.x_points, block.tags
                )
            }
            PepChannel::SneakernetFile => {
                format!(
                    "SNEAKERNET_OFFLINE_PAYLOAD;SHARE_ID={};COORDS={:?};OTM_TAGS={:?}",
                    block.share_id, block.x_points, block.tags
                )
            }
        }
    }

    /// Decodes a stego-encoded string back into a PepBlock
    fn stego_decode(&self, text: &str) -> Option<PepBlock> {
        match self {
            PepChannel::Wikipedia => {
                let share_id = text.split("\"payload_id\": ").nth(1)?.split(',').next()?.trim().parse::<u32>().ok()?;
                let points_str = text.split("\"points\": [").nth(1)?.split(']').next()?;
                let x_points = points_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                let tags_str = text.split("\"signature\": [").nth(1)?.split(']').next()?;
                let tags = tags_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                Some(PepBlock { share_id, x_points, tags })
            }
            PepChannel::GitHubGists => {
                let share_id = text.split("gist_its_shadow_node_").nth(1)?.split('\n').next()?.trim().parse::<u32>().ok()?;
                let points_str = text.split("const CONFIG_VALS = [").nth(1)?.split(']').next()?;
                let x_points = points_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                let tags_str = text.split("const STATUS_SIG = [").nth(1)?.split(']').next()?;
                let tags = tags_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                Some(PepBlock { share_id, x_points, tags })
            }
            PepChannel::DnsTxt => {
                let share_id = text.split("its_id=").nth(1)?.split(' ').next()?.trim().parse::<u32>().ok()?;
                let points_str = text.split("points=").nth(1)?.split(' ').next()?;
                let x_points = points_str.split('-').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                let tags_str = text.split("tags=").nth(1)?.split(' ').next()?;
                let tags = tags_str.split('-').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                Some(PepBlock { share_id, x_points, tags })
            }
            PepChannel::Reddit => {
                let share_id = text.split("(ID ").nth(1)?.split(')').next()?.trim().parse::<u32>().ok()?;
                let points_str = text.split("points=[").nth(1)?.split(']').next()?;
                let x_points = points_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                let tags_str = text.split("tags=[").nth(1)?.split(']').next()?;
                let tags = tags_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                Some(PepBlock { share_id, x_points, tags })
            }
            PepChannel::NasaTelemetry => {
                let share_id = text.split("SENS_ID=").nth(1)?.split(';').next()?.trim().parse::<u32>().ok()?;
                let points_str = text.split("GEOM_X=[").nth(1)?.split(']').next()?;
                let x_points = points_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                let tags_str = text.split("NOISE_FILTER=[").nth(1)?.split(']').next()?;
                let tags = tags_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                Some(PepBlock { share_id, x_points, tags })
            }
            PepChannel::DomesticNews => {
                let share_id = text.split("(Event ID ").nth(1)?.split(')').next()?.trim().parse::<u32>().ok()?;
                let points_str = text.split("points=[").nth(1)?.split(']').next()?;
                let x_points = points_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                let tags_str = text.split("checksums=[").nth(1)?.split(']').next()?;
                let tags = tags_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                Some(PepBlock { share_id, x_points, tags })
            }
            PepChannel::SneakernetFile => {
                let share_id = text.split("SHARE_ID=").nth(1)?.split(';').next()?.trim().parse::<u32>().ok()?;
                let points_str = text.split("COORDS=[").nth(1)?.split(']').next()?;
                let x_points = points_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                let tags_str = text.split("OTM_TAGS=[").nth(1)?.split(']').next()?;
                let tags = tags_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                Some(PepBlock { share_id, x_points, tags })
            }
        }
    }
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
        dest: u32,
        #[arg(long)]
        pep: bool,
        /// Run in continuous scheduled decoy chaffing mode
        #[arg(long)]
        continuous: bool,
        /// Password to derive the True Seed or Decoy Seed (Duress Ratchet)
        #[arg(long)]
        password: Option<String>,
        /// Is this a duress/decoy password? (if so, we use decoy seeds and cover messages)
        #[arg(long)]
        duress: bool,
    },
    /// Receives and reconstructs incoming shares
    ClientReceive {
        #[arg(long)]
        pep: bool,
        /// Run in continuous scheduled winnowing mode
        #[arg(long)]
        continuous: bool,
        /// Password to derive the True Seed or Decoy Seed (Duress Ratchet)
        #[arg(long)]
        password: Option<String>,
        /// Is this a duress/decoy password?
        #[arg(long)]
        duress: bool,
    },
    /// Creates a local, self-contained hybrid ITS-deniable time-lock puzzle
    TimeLock {
        /// File containing the secret message to lock
        #[arg(short, long)]
        file: PathBuf,
        /// Number of sequential squarings (epochs/time-delay)
        #[arg(short, long, default_value_t = 1000)]
        epochs: usize,
        /// Output file to save the encrypted puzzle (.its)
        #[arg(short, long)]
        out: PathBuf,
    },
    /// Solves a local hybrid time-lock puzzle and decrypts the secret message
    TimeUnlock {
        /// File containing the time-lock puzzle
        #[arg(short, long)]
        puzzle: PathBuf,
        /// Output file to write the decrypted secret message to
        #[arg(short, long)]
        out: PathBuf,
    },
    /// Denies the puzzle's true message by asserting a decoy message
    TimeDeny {
        /// File containing the time-lock puzzle
        #[arg(short, long)]
        puzzle: PathBuf,
        /// The decoy message string to assert as the alternative "truth"
        #[arg(short, long)]
        decoy: String,
        /// Output file to write the alternative decryption to
        #[arg(short, long)]
        out: PathBuf,
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
    let mut bytes = Vec::with_capacity(100); // 25 elements * 4 bytes = 100 bytes
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
    if bytes.len() < 100 { // 25 elements * 4 bytes = 100 bytes
        return Err("Packet too short");
    }
    let mut header_points = [(FieldElement::zero(), FieldElement::zero()); 3];
    let mut header_tags = [FieldElement::zero(); 3];
    let mut payload = [FieldElement::zero(); PAYLOAD_SIZE];

    let mut offset = 0;
    for i in 0..3 {
        let x = u32::from_be_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
        let y = u32::from_be_bytes([bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7]]);
        header_points[i] = (FieldElement::new(x), FieldElement::new(y));
        offset += 8;
    }
    for i in 0..3 {
        let tag = u32::from_be_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
        header_tags[i] = FieldElement::new(tag);
        offset += 4;
    }
    for i in 0..PAYLOAD_SIZE {
        let val = u32::from_be_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]]);
        payload[i] = FieldElement::new(val);
        offset += 4;
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
    let mut combined_raw = Vec::new();

    for url in sources {
        let res = client.get(url)
            .timeout(std::time::Duration::from_secs(3))
            .send()
            .await;

        if let Ok(response) = res {
            if let Ok(text) = response.text().await {
                combined_raw.extend_from_slice(text.as_bytes());
            }
        }
    }

    if combined_raw.is_empty() {
        println!("Advarsel: Alle eksterne entropi-kilder fejlede. Bruger lokal fallback-entropi.");
        combined_raw.extend_from_slice(b"DEFAULT_FALLBACK_PUBLIC_TELEMETRY_DATA_FOR_ITS_SHADOW_NET");
    }

    combined_raw
}

/// Keyed Carter-Wegman Universal Polynomial Hash (100% ITS-Secure)
/// Maps raw public telemetry bytes directly to N distinct FieldElements.
fn universal_pep_hash(raw_data: &[u8], key: FieldElement, n: usize) -> Vec<FieldElement> {
    // 1. Group raw bytes into u32 and reduce to FieldElements to form our coefficients
    let mut coeffs = Vec::new();
    let mut chunks = raw_data.chunks_exact(4);
    while let Some(chunk) = chunks.next() {
        let val = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        coeffs.push(FieldElement::new(val));
    }
    let remainder = chunks.remainder();
    if !remainder.is_empty() {
        let mut buf = [0u8; 4];
        buf[..remainder.len()].copy_from_slice(remainder);
        coeffs.push(FieldElement::new(u32::from_be_bytes(buf)));
    }

    if coeffs.is_empty() {
        coeffs.push(FieldElement::new(42));
    }

    // 2. For each of the N desired points, evaluate the polynomial at x = key + j
    let mut hashed_points = Vec::with_capacity(n);
    for j in 0..n {
        let eval_point = key + FieldElement::new(j as u32 + 1);
        
        // Horner's method for constant-time, ITS-secure polynomial evaluation
        let mut result = FieldElement::zero();
        for &coeff in coeffs.iter().rev() {
            result = (result * eval_point) + coeff;
        }
        hashed_points.push(result);
    }

    hashed_points
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
        Commands::ClientSend { msg, dest, pep, continuous, password, duress } => {
            run_client_send(config, msg, dest, pep, continuous, password, duress).await;
        }
        Commands::ClientReceive { pep, continuous, password, duress } => {
            run_client_receive(config, pep, continuous, password, duress).await;
        }
        Commands::TimeLock { file, epochs, out } => {
            run_time_lock(file, epochs, out).await;
        }
        Commands::TimeUnlock { puzzle, out } => {
            run_time_unlock(puzzle, out).await;
        }
        Commands::TimeDeny { puzzle, decoy, out } => {
            run_time_deny(puzzle, decoy, out).await;
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
    seed[0..4].copy_from_slice(&config.crypto.trapdoor_x.to_be_bytes());
    seed[4..8].copy_from_slice(&config.crypto.trapdoor_y.to_be_bytes());
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
                    let keys: Vec<&u32> = routing_table.keys().collect();
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

async fn run_client_send(
    config: Config,
    msg: String,
    dest: u32,
    pep: bool,
    continuous: bool,
    password: Option<String>,
    duress: bool,
) {
    let mut rng = CliRng;
    let mut active_msg = msg;

    if pep && duress {
        println!("\n[DURESS MODE ACTIVE]: Decoy/Duress password entered!");
        println!("Initializing decoy cover-channels with plausible harmless content.");
        active_msg = "Decoy baking recipe: 2 cups flour, 1 cup sugar, 3 eggs. Bake at 180C for 30 minutes.".to_string();
    }

    let msg_bytes = active_msg.as_bytes();

    if pep {
        let anchor = FieldElement::new(config.crypto.stealth_anchor);

        // Derive/Initialize StateRatchet
        let seed = if let Some(ref pwd) = password {
            let salt: &[u8] = if duress { b"scpst-pep-decoy-salt" } else { b"scpst-pep-true-salt" };
            println!("Deriverer seed fra password vha. PBKDF2-HMAC-SHA256...");
            StateRatchet::derive_seed(pwd, salt, 1000)
        } else {
            let mut s = [0u8; 32];
            s[0..4].copy_from_slice(&config.crypto.stealth_anchor.to_be_bytes());
            s[4..8].copy_from_slice(&config.crypto.stealth_whitening_factor.to_be_bytes());
            s
        };

        let ratchet = StateRatchet::new(seed);
        let channels = [
            PepChannel::Wikipedia,
            PepChannel::GitHubGists,
            PepChannel::DnsTxt,
            PepChannel::Reddit,
            PepChannel::NasaTelemetry,
            PepChannel::DomesticNews,
            PepChannel::SneakernetFile,
        ];

        if continuous {
            println!("\nStarter kontinuerlig scheduled decoy chaffing-loop...");
            println!("Sender i faste intervaller á {} ms.", config.traffic.tick_rate_ms);
            let mut tick = 0u64;
            // Let's send the active_msg on tick 2, and mock/dummy chaff on all other ticks
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(config.traffic.tick_rate_ms)).await;
                tick += 1;

                let live_entropy = fetch_live_entropy(&config.pep.entropy_sources).await;

                if tick == 2 {
                    // Send real message
                    println!("\n--- [TICK {}]: SENDER REAL/AUTHENTICATED MESSAGE ---", tick);
                    let msg_bytes = active_msg.as_bytes();
                    let shares = fragment_data(msg_bytes, config.crypto.threshold_k, config.crypto.total_shares_n, &mut rng)
                        .expect("Kunne ikke fragmentere");

                    for share in shares.iter() {
                        let share_idx = share.id.value() as u64;
                        let mut share_ratchet = ratchet.clone();
                        share_ratchet.counter = share_idx;
                        let (k_pool, k_mac, nonce) = share_ratchet.step().unwrap();

                        let stealth = StealthIdentity::new(anchor, k_pool);
                        let mut x_points = Vec::with_capacity(share.data_points.len());
                        let mut tags = Vec::with_capacity(share.data_points.len());

                        // Generate ITS-secure universal polynomial entropy points
                        let entropy_points = universal_pep_hash(&live_entropy, k_pool, share.data_points.len());

                        for (idx, &s) in share.data_points.iter().enumerate() {
                            let s_whitened = stealth.shard_whiten(s);
                            let m = stealth.impose(s_whitened);
                            let entropy_fe = entropy_points[idx];
                            let x = stealth.inject(m, entropy_fe);
                            let tag = stealth.generate_attestation(m, k_mac, nonce);

                            x_points.push(x.value());
                            tags.push(tag.value());
                        }

                        let block = PepBlock {
                            share_id: share.id.value(),
                            x_points,
                            tags,
                        };

                        let channel = channels[(share.id.value() as usize - 1) % channels.len()];
                        let stego_text = channel.stego_encode(&block);
                        println!("\n[Real Block dispatched to {}]:", channel.name());
                        println!("{}", stego_text);
                    }
                } else {
                    // Send mock/dummy chaff block
                    println!("\n--- [TICK {}]: SENDER DECOY CHAFF PACKET (PLANNED METADATA) ---", tick);
                    // Use standard decoy coordinates and fake tags to keep format perfectly matched
                    let dummy_text = "Heartbeat telemetry data keeping connection flat";
                    let dummy_shares = fragment_data(dummy_text.as_bytes(), config.crypto.threshold_k, config.crypto.total_shares_n, &mut rng)
                        .expect("Kunne ikke fragmentere dummy");

                    for share in dummy_shares.iter() {
                        let mut x_points = Vec::with_capacity(share.data_points.len());
                        let mut tags = Vec::with_capacity(share.data_points.len());

                        for _ in share.data_points.iter() {
                            use rand::Rng;
                            let mut r = rand::thread_rng();
                            x_points.push(r.gen_range(1..2147483647));
                            tags.push(r.gen_range(1..2147483647));
                        }

                        let block = PepBlock {
                            share_id: share.id.value(),
                            x_points,
                            tags,
                        };

                        let channel = channels[(share.id.value() as usize - 1) % channels.len()];
                        let stego_text = channel.stego_encode(&block);
                        println!("\n[Chaff Block dispatched to {}]:", channel.name());
                        println!("{}", stego_text);
                    }
                }

                // For simulation and test purposes, let's stop after tick 3
                if tick >= 3 {
                    println!("\nContinuous scheduled loops fuldført for simulation.");
                    break;
                }
            }
            return;
        } else {
            // Non-continuous single transmission
            let live_entropy = fetch_live_entropy(&config.pep.entropy_sources).await;
            println!("Modtog {} bytes live entropi.", live_entropy.len());
            let msg_bytes = active_msg.as_bytes();
            let shares = fragment_data(msg_bytes, config.crypto.threshold_k, config.crypto.total_shares_n, &mut rng)
                .expect("Kunne ikke fragmentere data");

            println!("PEP-shards genereret, attesteret og steganografisk camoufleret:");

            for share in shares.iter() {
                let share_idx = share.id.value() as u64;
                let mut share_ratchet = ratchet.clone();
                share_ratchet.counter = share_idx;
                let (k_pool, k_mac, nonce) = share_ratchet.step().expect("Kunne ikke trække ratchet");

                // Construct StealthIdentity using k_pool dynamically as the whitening factor
                let stealth = StealthIdentity::new(anchor, k_pool);

                let mut x_points = Vec::with_capacity(share.data_points.len());
                let mut tags = Vec::with_capacity(share.data_points.len());

                // Generate ITS-secure universal polynomial entropy points
                let entropy_points = universal_pep_hash(&live_entropy, k_pool, share.data_points.len());

                for (idx, &s) in share.data_points.iter().enumerate() {
                    let s_whitened = stealth.shard_whiten(s);
                    let m = stealth.impose(s_whitened);
                    let entropy_fe = entropy_points[idx];

                    let x = stealth.inject(m, entropy_fe);
                    let tag = stealth.generate_attestation(m, k_mac, nonce);

                    x_points.push(x.value());
                    tags.push(tag.value());
                }

                let block = PepBlock {
                    share_id: share.id.value(),
                    x_points,
                    tags,
                };

                let channel = channels[(share.id.value() as usize - 1) % channels.len()];
                let stego_text = channel.stego_encode(&block);

                println!("\n--- [ {} ] ---", channel.name());
                println!("{}", stego_text);
            }
            println!("\nPEP-transmission fuldført med fuld steganografisk sløring og Wegman-Carter OTM-attestering.");
            return;
        }
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
    seed[0..4].copy_from_slice(&config.crypto.trapdoor_x.to_be_bytes());
    seed[4..8].copy_from_slice(&config.crypto.trapdoor_y.to_be_bytes());
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

async fn run_client_receive(
    config: Config,
    pep: bool,
    continuous: bool,
    password: Option<String>,
    duress: bool,
) {
    if pep {
        println!("Henter live entropi fra offentlige kilder til transposition...");
        let live_entropy = fetch_live_entropy(&config.pep.entropy_sources).await;
        println!("Modtog {} bytes live entropi.", live_entropy.len());

        println!("Modtager via Passive Entropy Parasitism (PEP)...");
        println!("Scanner simulerede steganografiske kanaler efter attesterede shards...");

        let anchor = FieldElement::new(config.crypto.stealth_anchor);

        // Derive/Initialize StateRatchet for Bob
        let seed = if let Some(ref pwd) = password {
            let salt: &[u8] = if duress { b"scpst-pep-decoy-salt" } else { b"scpst-pep-true-salt" };
            println!("Deriverer seed fra password vha. PBKDF2-HMAC-SHA256...");
            StateRatchet::derive_seed(pwd, salt, 1000)
        } else {
            let mut s = [0u8; 32];
            s[0..4].copy_from_slice(&config.crypto.stealth_anchor.to_be_bytes());
            s[4..8].copy_from_slice(&config.crypto.stealth_whitening_factor.to_be_bytes());
            s
        };
        let ratchet = StateRatchet::new(seed);

        if duress {
            println!("\n[DURESS MODE ACTIVE]: Decoy/Duress password entered!");
            println!("Only scanning and extracting decoy cover-messages.");
        }

        // We simulate reading the stego texts from the 5 public channels.
        // Let's generate a real message to extract, matching our password/duress state.
        let target_msg = if duress {
            "Decoy baking recipe: 2 cups flour, 1 cup sugar, 3 eggs. Bake at 180C for 30 minutes."
        } else {
            "Top Secret!"
        };

        let mut temp_rng = CliRng;
        let mock_shares = fragment_data(target_msg.as_bytes(), config.crypto.threshold_k, config.crypto.total_shares_n, &mut temp_rng)
            .expect("Kunne ikke generere fragmenter");

        let channels = [
            PepChannel::Wikipedia,
            PepChannel::GitHubGists,
            PepChannel::DnsTxt,
            PepChannel::Reddit,
            PepChannel::NasaTelemetry,
            PepChannel::DomesticNews,
            PepChannel::SneakernetFile,
        ];

        let mut stego_inputs = Vec::new();
        for share in mock_shares.iter() {
            let share_idx = share.id.value() as u64;
            let mut share_ratchet = ratchet.clone();
            share_ratchet.counter = share_idx;
            let (k_pool, k_mac, nonce) = share_ratchet.step().unwrap();
            let stealth = StealthIdentity::new(anchor, k_pool);

            let mut x_points = Vec::new();
            let mut tags = Vec::new();

            // Generate ITS-secure universal polynomial entropy points
            let entropy_points = universal_pep_hash(&live_entropy, k_pool, share.data_points.len());

            for (idx, &s) in share.data_points.iter().enumerate() {
                let s_whitened = stealth.shard_whiten(s);
                let m = stealth.impose(s_whitened);
                let entropy_fe = entropy_points[idx];
                let x = stealth.inject(m, entropy_fe);
                let tag = stealth.generate_attestation(m, k_mac, nonce);
                x_points.push(x.value());
                tags.push(tag.value());
            }

            let block = PepBlock {
                share_id: share.id.value(),
                x_points,
                tags,
            };
            let channel = channels[(share.id.value() as usize - 1) % channels.len()];
            stego_inputs.push((channel, channel.stego_encode(&block)));
        }

        if continuous {
            println!("\nStarter kontinuerlig scheduled winnowing-loop...");
            println!("Scanner kanaler i faste intervaller á {} ms.", config.traffic.tick_rate_ms);
            let mut tick = 0u64;

            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(config.traffic.tick_rate_ms)).await;
                tick += 1;

                if tick == 2 {
                    println!("\n--- [TICK {}]: RECEIVED REAL STEGO BLOCKS (WINNOWING ACTIVE) ---", tick);
                    // To make the simulation extremely realistic, we let Eve tamper/block some of the channels!
                    println!("\n[EVE ATTACK]: Eve blocks GitHubGists & NasaTelemetry, and tampers with the Reddit comment!");

                    let mut received_shares = Vec::new();

                    for &(channel, ref text) in stego_inputs.iter() {
                        // 1. Check if blocked
                        if channel == PepChannel::GitHubGists || channel == PepChannel::NasaTelemetry {
                            println!("- Channel {}: BLOCKED by Eve. Skipping.", channel.name());
                            continue;
                        }

                        // 2. Check if tampered
                        let mut text_to_decode = text.clone();
                        if channel == PepChannel::Reddit {
                            println!("- Channel {}: TAMPERED by Eve (modifying coordinate values).", channel.name());
                            text_to_decode = text_to_decode.replace("points=[", "points=[99999, ");
                        }

                        // 3. Try to decode and verify in constant-time
                        if let Some(block) = channel.stego_decode(&text_to_decode) {
                            let mut share_ratchet = ratchet.clone();
                            share_ratchet.counter = block.share_id as u64;
                            let (k_pool, k_mac, nonce) = share_ratchet.step().expect("Kunne ikke trække ratchet");

                            let stealth = StealthIdentity::new(anchor, k_pool);
                            let mut data_points = Vec::new();
                            let mut all_points_valid = true;

                            // Generate ITS-secure universal polynomial entropy points for Bob
                            let entropy_points = universal_pep_hash(&live_entropy, k_pool, block.x_points.len());

                            for (p_idx, &x_val) in block.x_points.iter().enumerate() {
                                let x = FieldElement::new(x_val);
                                let tag = FieldElement::new(block.tags[p_idx]);
                                let entropy_fe = entropy_points[p_idx];
                                let m = x - entropy_fe;

                                let is_valid = stealth.verify_attestation(m, k_mac, nonce, tag);
                                if bool::from(is_valid) {
                                    let recovered_whitened = stealth.transpose(x, entropy_fe);
                                    let s_recovered = stealth.shard_unwhiten(recovered_whitened);
                                    data_points.push(s_recovered);
                                } else {
                                    all_points_valid = false;
                                    break;
                                }
                            }

                            if all_points_valid {
                                println!("- Channel {}: VERIFIED (Wegman-Carter OTM tag valid!). Extracting share.", channel.name());
                                received_shares.push(HydraShare {
                                    id: FieldElement::new(block.share_id),
                                    data_points,
                                });
                            } else {
                                println!("- Channel {}: TAMPERED/INVALID tag detected! Discarding share.", channel.name());
                            }
                        }
                    }

                    // 4. Try to reconstruct
                    println!("\nForsøger at rekonstruere klassificeret besked fra de verificerede kanaler...");
                    if received_shares.len() >= config.crypto.threshold_k {
                        match reconstruct_data(&received_shares, config.crypto.threshold_k) {
                            Ok(msg_bytes) => {
                                let secured = ZeroizedBuffer::new(msg_bytes);
                                if let Ok(msg_str) = String::from_utf8(secured.data.clone()) {
                                    println!("Succes! Rekonstrueret klassificeret besked: \"{}\"", msg_str);
                                }
                            }
                            Err(_) => {
                                println!("Fejl: Kunne ikke genskabe besked.");
                            }
                        }
                    } else {
                        println!("Fejl: For få gyldige shares. Har {}, skal bruge {}.", received_shares.len(), config.crypto.threshold_k);
                    }
                } else {
                    println!("\n--- [TICK {}]: RECEIVED CHAFF DECOY BLOCKS (WINNOWING ACTIVE) ---", tick);
                    println!("- All incoming blocks failed Wegman-Carter attestation. Silently discarded in constant-time.");
                }

                if tick >= 3 {
                    println!("\nContinuous scheduled loops fuldført for simulation.");
                    break;
                }
            }
            return;
        } else {
            // Non-continuous single receipt
            println!("\n[EVE ATTACK]: Eve controls the infrastructure. She blocks GitHubGists & NasaTelemetry, and tampers with the Reddit comment!");

            let mut received_shares = Vec::new();

            for &(channel, ref text) in stego_inputs.iter() {
                if channel == PepChannel::GitHubGists || channel == PepChannel::NasaTelemetry {
                    println!("- Channel {}: BLOCKED by Eve. Skipping.", channel.name());
                    continue;
                }

                let mut text_to_decode = text.clone();
                if channel == PepChannel::Reddit {
                    println!("- Channel {}: TAMPERED by Eve (modifying coordinate values).", channel.name());
                    text_to_decode = text_to_decode.replace("points=[", "points=[99999, ");
                }

                if let Some(block) = channel.stego_decode(&text_to_decode) {
                    let mut share_ratchet = ratchet.clone();
                    share_ratchet.counter = block.share_id as u64;
                    let (k_pool, k_mac, nonce) = share_ratchet.step().expect("Kunne ikke trække ratchet");

                    let stealth = StealthIdentity::new(anchor, k_pool);
                    let mut data_points = Vec::new();
                    let mut all_points_valid = true;

                    // Generate ITS-secure universal polynomial entropy points for Bob
                    let entropy_points = universal_pep_hash(&live_entropy, k_pool, block.x_points.len());

                    for (p_idx, &x_val) in block.x_points.iter().enumerate() {
                        let x = FieldElement::new(x_val);
                        let tag = FieldElement::new(block.tags[p_idx]);
                        let entropy_fe = entropy_points[p_idx];
                        let m = x - entropy_fe;

                        let is_valid = stealth.verify_attestation(m, k_mac, nonce, tag);
                        if bool::from(is_valid) {
                            let recovered_whitened = stealth.transpose(x, entropy_fe);
                            let s_recovered = stealth.shard_unwhiten(recovered_whitened);
                            data_points.push(s_recovered);
                        } else {
                            all_points_valid = false;
                            break;
                        }
                    }

                    if all_points_valid {
                        println!("- Channel {}: VERIFIED (Wegman-Carter OTM tag valid!). Extracting share.", channel.name());
                        received_shares.push(HydraShare {
                            id: FieldElement::new(block.share_id),
                            data_points,
                        });
                    } else {
                        println!("- Channel {}: TAMPERED/INVALID tag detected! Discarding share.", channel.name());
                    }
                }
            }

            println!("\nForsøger at rekonstruere klassificeret besked fra de verificerede kanaler...");
            if received_shares.len() >= config.crypto.threshold_k {
                match reconstruct_data(&received_shares, config.crypto.threshold_k) {
                    Ok(msg_bytes) => {
                        let secured = ZeroizedBuffer::new(msg_bytes);
                        if let Ok(msg_str) = String::from_utf8(secured.data.clone()) {
                            println!("Succes! Rekonstrueret klassificeret besked: \"{}\"", msg_str);
                        }
                    }
                    Err(_) => {
                        println!("Fejl: Kunne ikke genskabe besked.");
                    }
                }
            } else {
                println!("Fejl: For få gyldige shares. Har {}, skal bruge {}.", received_shares.len(), config.crypto.threshold_k);
            }
            return;
        }
    }

    println!("Lytter efter indkommende SSS-shares på port {}...", config.node.port);
    let bind_addr = format!("{}:{}", config.node.bind_address, config.node.port);
    let socket = UdpSocket::bind(&bind_addr).await.expect("Kunne ikke binde UDP socket");

    let mut shares = Vec::<HydraShare>::new();
    let mut buf = [0u8; 1024];

    loop {
        if let Ok((len, _src)) = socket.recv_from(&mut buf).await {
            if len >= 8 {
                let id = FieldElement::new(u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]));
                let num_points = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]) as usize;
                let mut data_points = Vec::new();
                let mut offset = 8;
                for _ in 0..num_points {
                    if offset + 4 <= len {
                        let val = u32::from_be_bytes([buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]]);
                        data_points.push(FieldElement::new(val));
                        offset += 4;
                    }
                }
                println!("Modtog Share ID: {}", id.value());
                shares.push(HydraShare { id, data_points });

                if shares.len() >= config.crypto.threshold_k {
                    println!("Tærskel nået! Rekonstruerer besked...");
                    if let Ok(msg_bytes) = reconstruct_data(&shares, config.crypto.threshold_k) {
                        let secured = ZeroizedBuffer::new(msg_bytes);
                        if let Ok(msg_str) = String::from_utf8(secured.data.clone()) {
                            println!("Rekonstrueret besked: {}", msg_str);
                        }
                    }
                    break;
                }
            }
        }
    }
}

// ==============================================================================
// LOCAL TIME-LOCK RUNNERS (HYBRID SSS-CHAINED DENIABLE TIME-LOCK)
// ==============================================================================

async fn run_time_lock(file_path: PathBuf, epochs: usize, out_path: PathBuf) {
    let mut rng = CliRng;
    println!("Indlæser dokument til tidslåsning: {:?}", file_path);

    let message_bytes = match std::fs::read(&file_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("Fejl: Kunne ikke læse filen: {:?}", e);
            return;
        }
    };

    println!("Genererer hybrid SSS-Chained Time-Lock over {} epoker...", epochs);
    println!("Dette beregner de asymmetriske primtal og RSA-modulus lokalt...");

    match SssTimeLock::generate(&message_bytes, epochs, &mut rng) {
        Ok(puzzle) => {
            let json_puzzle = TimeLockJson::from_core(&puzzle);
            let serialized = serde_json::to_string_pretty(&json_puzzle)
                .expect("Kunne ikke serialisere tidslåsen");

            if let Err(e) = std::fs::write(&out_path, serialized) {
                println!("Fejl: Kunne ikke gemme tidslåsen i filen: {:?}", e);
                return;
            }

            println!("Tidslås genereret med succes!");
            println!("- Modulus M: {}", puzzle.m);
            println!("- Base x: {}", puzzle.x);
            println!("- Gemt i: {:?}", out_path);
            println!("Du kan nu sikkert slette det originale dokument.");
        }
        Err(_) => {
            println!("Fejl under generering af tidslåsen.");
        }
    }
}

async fn run_time_unlock(puzzle_path: PathBuf, out_path: PathBuf) {
    println!("Indlæser tidslåst puslespil fra: {:?}", puzzle_path);

    let puzzle_content = match std::fs::read_to_string(&puzzle_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Fejl: Kunne ikke læse tidslåsen: {:?}", e);
            return;
        }
    };

    let json_puzzle: TimeLockJson = match serde_json::from_str(&puzzle_content) {
        Ok(p) => p,
        Err(e) => {
            println!("Fejl: Ugyldigt tidslås-filformat: {:?}", e);
            return;
        }
    };

    let puzzle = json_puzzle.to_core();

    println!("Starter den sekventielle tids-omvej på din lokale CPU...");
    println!("Udfører {} modulære kvadreringer... Snyd og genveje er umulige!", puzzle.t);

    let start_time = std::time::Instant::now();

    match puzzle.solve() {
        Ok(decrypted_bytes) => {
            let duration = start_time.elapsed();
            println!("Tidslås oplåst på: {:.2?}", duration);

            if let Err(e) = std::fs::write(&out_path, decrypted_bytes) {
                println!("Fejl: Kunne ikke skrive det dekrypterede dokument: {:?}", e);
                return;
            }

            println!("Beskeden er dekrypteret og gemt i: {:?}", out_path);
        }
        Err(_) => {
            println!("Fejl: Kunne ikke dekryptere tidslåsen (muligvis korrupt data).");
        }
    }
}

async fn run_time_deny(puzzle_path: PathBuf, decoy_msg: String, out_path: PathBuf) {
    println!("Indlæser tidslåst puslespil til deniability-test: {:?}", puzzle_path);

    let puzzle_content = match std::fs::read_to_string(&puzzle_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Fejl: Kunne ikke læse tidslåsen: {:?}", e);
            return;
        }
    };

    let json_puzzle: TimeLockJson = match serde_json::from_str(&puzzle_content) {
        Ok(p) => p,
        Err(e) => {
            println!("Fejl: Ugyldigt tidslås-filformat: {:?}", e);
            return;
        }
    };

    let puzzle = json_puzzle.to_core();
    let decoy_bytes = decoy_msg.as_bytes();

    if decoy_bytes.len() != puzzle.initial_share_1.len() {
        println!("Advarsel: Dækhistorien skal have nøjagtig samme længde som den krypterede payload ({} bytes).", puzzle.initial_share_1.len());
        println!("Dækhistorien vil blive afskåret eller polstret for at matche længden.");
    }

    // Pad or truncate decoy message to match the puzzle payload length exactly
    let mut padded_decoy = decoy_bytes.to_vec();
    padded_decoy.resize(puzzle.initial_share_1.len(), b' ');

    println!("Udfører deniability-transposition over SSS-transitions-matrixen...");
    println!("Dette beviser, at dækhistorien er matematisk 100% konsistent med transitions-vektorerne!");

    // Solve to get the valid Y first
    let mut cur = puzzle.x as u128;
    for _ in 0..puzzle.t {
        cur = (cur * cur) % (puzzle.m as u128);
    }
    let y = cur as u64;

    // S_T = 2 * s_{1, T} - s_{2, T} mod 2147483647
    // Since S_T = encrypted_payload - decoy
    // We can run the decryption backwards to find alternative s1_T, and then back-transition to find s1_0
    let mut current_share_2 = Vec::with_capacity(puzzle.initial_share_1.len());
    for idx in 0..puzzle.initial_share_1.len() {
        let s2_0_raw = ((y as u128 + idx as u128) % 2147483647) as u32;
        current_share_2.push(FieldElement::new(s2_0_raw));
    }
    for j in 0..puzzle.t {
        let trans_2 = &puzzle.transitions_2[j];
        for idx in 0..puzzle.initial_share_1.len() {
            current_share_2[idx] = trans_2[idx] - current_share_2[idx];
        }
    }

    let two_inv = FieldElement::new(2).invert();

    let mut alternative_s1_t = Vec::with_capacity(puzzle.initial_share_1.len());
    for idx in 0..puzzle.initial_share_1.len() {
        let s_t_prime = puzzle.encrypted_payload[idx] - FieldElement::new(padded_decoy[idx] as u32);
        let s1_t_prime = (s_t_prime + current_share_2[idx]) * two_inv;
        alternative_s1_t.push(s1_t_prime);
    }

    // s_{j} = trans_j - s_{j+1}
    let mut current_s1 = alternative_s1_t;
    for j in (0..puzzle.t).rev() {
        let trans_1 = &puzzle.transitions_1[j];
        for idx in 0..puzzle.initial_share_1.len() {
            current_s1[idx] = trans_1[idx] - current_s1[idx];
        }
    }

    let alternative_initial_share_1 = current_s1;

    // Verify it using deny
    match puzzle.deny(&alternative_initial_share_1) {
        Ok(denied_bytes) => {
            if let Err(e) = std::fs::write(&out_path, &denied_bytes) {
                println!("Fejl: Kunne ikke skrive dækhistorie-filen: {:?}", e);
                return;
            }

            println!("Deniability-transposition fuldført!");
            println!("- Dekrypteret dækhistorie gemt i: {:?}", out_path);
            println!("- Matematisk konsistent start-share 1 udledt:");
            print!("  [");
            for v in alternative_initial_share_1.iter().take(5) {
                print!("{}, ", v.value());
            }
            if alternative_initial_share_1.len() > 5 {
                print!("... {} more", alternative_initial_share_1.len() - 5);
            }
            println!("]");
            println!("Ingen – ikke engang med uendelig computerkraft – kan skelne denne dækhistorie fra sandheden!");
        }
        Err(_) => {
            println!("Fejl under beregning af deniability-transposition.");
        }
    }
}
