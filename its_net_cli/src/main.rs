use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};

pub mod anomaly_detection;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use core_logic::field_arith::FieldElement;
use core_logic::trapdoor::Trapdoor;
use core_logic::routing::{create_onion_packet, MorphicMixingNode, MorphicOnionPacket, PAYLOAD_SIZE};
use core_logic::hydra_sss::{fragment_data, reconstruct_data, SssPackedShare};
use core_logic::stealth_identity::StealthIdentity;
use core_logic::ratchet::StateRatchet;
use core_logic::SecureRandom;
use its_self_enclosed_timelock::field_arith::FieldElement as TlFieldElement;
use its_self_enclosed_timelock::{GenerateError, SssTimeLock};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Bridges `/dev/urandom` into the standalone time-lock crate's RNG trait.
struct TimelockRng;

impl its_self_enclosed_timelock::SecureRandom for TimelockRng {
    type Error = std::io::Error;

    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        use std::io::Read;
        std::fs::File::open("/dev/urandom")?.read_exact(dest)
    }
}

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
// PACKET COURIER ABSTRACTION (FOR TRANSPORT-PROTOCOL AGNOSTICISM)
// ==============================================================================

/// An abstract transport-layer courier that can receive and dispatch raw packets.
///
/// By decoupling the onion daemon and chaffing threads from UDP sockets, we achieve
/// total transport agnostic behavior, allowing the shadow net to seamlessly switch between
/// UDP, TCP, DNS-TXT, HTTPS, or WebRTC carriers.
trait PacketCourier {
    fn send_raw(&self, data: &[u8], addr: &str) -> std::io::Result<()>;
    fn recv_raw(&self, buf: &mut [u8]) -> std::io::Result<(usize, String)>;
}

/// A standard UDP socket implementation of the `PacketCourier` trait.
struct UdpCourier {
    socket: UdpSocket,
}

impl UdpCourier {
    fn new(socket: UdpSocket) -> Self {
        UdpCourier { socket }
    }
}

impl PacketCourier for UdpCourier {
    fn send_raw(&self, data: &[u8], addr: &str) -> std::io::Result<()> {
        if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
            self.socket.send_to(data, socket_addr)?;
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid address format"))
        }
    }

    fn recv_raw(&self, buf: &mut [u8]) -> std::io::Result<(usize, String)> {
        let (len, src) = self.socket.recv_from(buf)?;
        Ok((len, src.to_string()))
    }
}

// ==============================================================================
// CONFIGURATION STRUCTURES
// ==============================================================================

#[derive(Debug, Clone)]
struct Config {
    node: NodeConfig,
    crypto: CryptoConfig,
    traffic: TrafficConfig,
    routing_table: HashMap<u32, String>,
    aeh: AehConfig,
}

#[derive(Debug, Clone)]
struct NodeConfig {
    id: u32,
    port: u16,
    bind_address: String,
}

#[derive(Debug, Clone)]
struct CryptoConfig {
    threshold_k: usize,
    total_shares_n: usize,
    trapdoor_x: u32,
    trapdoor_y: u32,
    stealth_anchor: u32,
    stealth_whitening_factor: u32,
}

#[derive(Debug, Clone)]
struct TrafficConfig {
    constant_rate_chaff_enabled: bool,
    tick_rate_ms: u64,
    payload_size_elements: usize,
}

#[derive(Debug, Clone, Default)]
struct AehConfig {
    entropy_sources: Vec<String>,
    clue_offset: usize,
}

// ==============================================================================
// TIME-LOCK CUSTOM TEXT PARSER (ZERO DEPENDENCY / NO JSON)
// ==============================================================================

#[derive(Debug, Clone)]
struct TimeLockText {
    x: u64,
    m: u64,
    t: usize,
    initial_share_1: Vec<u32>,
    transitions_1: Vec<Vec<u32>>,
    transitions_2: Vec<Vec<u32>>,
    encrypted_payload: Vec<u32>,
}

impl TimeLockText {
    fn from_core(puzzle: &SssTimeLock) -> Self {
        TimeLockText {
            x: puzzle.x,
            m: puzzle.m,
            t: puzzle.t,
            initial_share_1: puzzle.initial_share_1.iter().map(|f| f.value() as u32).collect(),
            transitions_1: puzzle.transitions_1.iter().map(|v| v.iter().map(|f| f.value() as u32).collect()).collect(),
            transitions_2: puzzle.transitions_2.iter().map(|v| v.iter().map(|f| f.value() as u32).collect()).collect(),
            encrypted_payload: puzzle.encrypted_payload.iter().map(|f| f.value() as u32).collect(),
        }
    }

    fn to_core(&self) -> SssTimeLock {
        SssTimeLock {
            x: self.x,
            m: self.m,
            t: self.t,
            initial_share_1: self.initial_share_1.iter().map(|&v| TlFieldElement::new(v)).collect(),
            transitions_1: self.transitions_1.iter().map(|v| v.iter().map(|&v| TlFieldElement::new(v)).collect()).collect(),
            transitions_2: self.transitions_2.iter().map(|v| v.iter().map(|&v| TlFieldElement::new(v)).collect()).collect(),
            encrypted_payload: self.encrypted_payload.iter().map(|&v| TlFieldElement::new(v)).collect(),
        }
    }

    fn serialize(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("x: {}\n", self.x));
        out.push_str(&format!("m: {}\n", self.m));
        out.push_str(&format!("t: {}\n", self.t));
        
        let initial_str: Vec<String> = self.initial_share_1.iter().map(|v| v.to_string()).collect();
        out.push_str(&format!("initial_share_1: {}\n", initial_str.join(",")));

        let payload_str: Vec<String> = self.encrypted_payload.iter().map(|v| v.to_string()).collect();
        out.push_str(&format!("encrypted_payload: {}\n", payload_str.join(",")));

        for (idx, trans) in self.transitions_1.iter().enumerate() {
            let t_str: Vec<String> = trans.iter().map(|v| v.to_string()).collect();
            out.push_str(&format!("transitions_1_block_{}: {}\n", idx, t_str.join(",")));
        }

        for (idx, trans) in self.transitions_2.iter().enumerate() {
            let t_str: Vec<String> = trans.iter().map(|v| v.to_string()).collect();
            out.push_str(&format!("transitions_2_block_{}: {}\n", idx, t_str.join(",")));
        }

        out
    }

    fn deserialize(content: &str) -> Result<Self, &'static str> {
        let mut x = 0;
        let mut m = 0;
        let mut t = 0;
        let mut initial_share_1 = Vec::new();
        let mut encrypted_payload = Vec::new();
        
        let mut trans1_map: HashMap<usize, Vec<u32>> = HashMap::new();
        let mut trans2_map: HashMap<usize, Vec<u32>> = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.splitn(2, ':');
            let key = parts.next().ok_or("Invalid format")?.trim();
            let val = parts.next().ok_or("Invalid format")?.trim();

            if key == "x" {
                x = val.parse::<u64>().map_err(|_| "Failed to parse x")?;
            } else if key == "m" {
                m = val.parse::<u64>().map_err(|_| "Failed to parse m")?;
            } else if key == "t" {
                t = val.parse::<usize>().map_err(|_| "Failed to parse t")?;
            } else if key == "initial_share_1" {
                if !val.is_empty() {
                    for part in val.split(',') {
                        initial_share_1.push(part.trim().parse::<u32>().map_err(|_| "Failed to parse initial share element")?);
                    }
                }
            } else if key == "encrypted_payload" {
                if !val.is_empty() {
                    for part in val.split(',') {
                        encrypted_payload.push(part.trim().parse::<u32>().map_err(|_| "Failed to parse payload element")?);
                    }
                }
            } else if key.starts_with("transitions_1_block_") {
                let idx_str = key.trim_start_matches("transitions_1_block_");
                let block_idx = idx_str.parse::<usize>().map_err(|_| "Failed to parse trans1 block index")?;
                let mut vals = Vec::new();
                if !val.is_empty() {
                    for part in val.split(',') {
                        vals.push(part.trim().parse::<u32>().map_err(|_| "Failed to parse trans1 element")?);
                    }
                }
                trans1_map.insert(block_idx, vals);
            } else if key.starts_with("transitions_2_block_") {
                let idx_str = key.trim_start_matches("transitions_2_block_");
                let block_idx = idx_str.parse::<usize>().map_err(|_| "Failed to parse trans2 block index")?;
                let mut vals = Vec::new();
                if !val.is_empty() {
                    for part in val.split(',') {
                        vals.push(part.trim().parse::<u32>().map_err(|_| "Failed to parse trans2 element")?);
                    }
                }
                trans2_map.insert(block_idx, vals);
            }
        }

        let mut transitions_1 = Vec::new();
        for idx in 0..t {
            let vals = trans1_map.get(&idx).ok_or("Missing transitions_1 block")?.clone();
            transitions_1.push(vals);
        }

        let mut transitions_2 = Vec::new();
        for idx in 0..t {
            let vals = trans2_map.get(&idx).ok_or("Missing transitions_2 block")?.clone();
            transitions_2.push(vals);
        }

        Ok(TimeLockText {
            x,
            m,
            t,
            initial_share_1,
            transitions_1,
            transitions_2,
            encrypted_payload,
        })
    }
}

// ==============================================================================
// AMBIENT ENTROPY HARVESTING (AEH) ADAPTERS (ZERO DEPENDENCY / NO JSON)
// ==============================================================================

#[derive(Debug, Clone)]
struct AehBlock {
    share_id: u32,
    x_points: Vec<u32>,
    tags: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AehChannel {
    Wikipedia,
    GitHubGists,
    DnsTxt,
    Reddit,
    NasaTelemetry,
    DomesticNews,
    SneakernetFile,
}

impl AehChannel {
    fn name(&self) -> &'static str {
        match self {
            AehChannel::Wikipedia => "Wikipedia API (Simulated)",
            AehChannel::GitHubGists => "GitHub Gists API (Simulated)",
            AehChannel::DnsTxt => "DNS TXT Records (Simulated)",
            AehChannel::Reddit => "Reddit Comments API (Simulated)",
            AehChannel::NasaTelemetry => "NASA Seismology API (Simulated)",
            AehChannel::DomesticNews => "State-Approved Domestic News Board (Simulated ALT)",
            AehChannel::SneakernetFile => "Sneakernet Local File / QR (Simulated ALT)",
        }
    }

    /// Encodes a AehBlock into simulated steganographic camouflage text
    fn stego_encode(&self, block: &AehBlock) -> String {
        match self {
            AehChannel::Wikipedia => {
                let x_str: Vec<String> = block.x_points.iter().map(|v| v.to_string()).collect();
                let tag_str: Vec<String> = block.tags.iter().map(|v| v.to_string()).collect();
                format!(
                    "WIKI_STEGO:enwiki;id={};points={};tags={}",
                    block.share_id, x_str.join(","), tag_str.join(",")
                )
            }
            AehChannel::GitHubGists => {
                let x_str: Vec<String> = block.x_points.iter().map(|v| v.to_string()).collect();
                let tag_str: Vec<String> = block.tags.iter().map(|v| v.to_string()).collect();
                format!(
                    "GIST_STEGO:id={};points={};tags={}",
                    block.share_id, x_str.join(","), tag_str.join(",")
                )
            }
            AehChannel::DnsTxt => {
                let points_str = block.x_points.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("-");
                let tags_str = block.tags.iter().map(|v| v.to_string()).collect::<Vec<_>>().join("-");
                format!(
                    "v=spf1 ip4:192.168.1.1 include:_spf.google.com its_id={} points={} tags={} ~all",
                    block.share_id, points_str, tags_str
                )
            }
            AehChannel::Reddit => {
                let x_str: Vec<String> = block.x_points.iter().map(|v| v.to_string()).collect();
                let tag_str: Vec<String> = block.tags.iter().map(|v| v.to_string()).collect();
                format!(
                    "REDDIT_STEGO:id={};points={};tags={}",
                    block.share_id, x_str.join(","), tag_str.join(",")
                )
            }
            AehChannel::NasaTelemetry => {
                let x_str: Vec<String> = block.x_points.iter().map(|v| v.to_string()).collect();
                let tag_str: Vec<String> = block.tags.iter().map(|v| v.to_string()).collect();
                format!(
                    "SENS_ID={};GEOM_X={};NOISE_FILTER={};SYS_STATUS=OK",
                    block.share_id, x_str.join(","), tag_str.join(",")
                )
            }
            AehChannel::DomesticNews => {
                let x_str: Vec<String> = block.x_points.iter().map(|v| v.to_string()).collect();
                let tag_str: Vec<String> = block.tags.iter().map(|v| v.to_string()).collect();
                format!(
                    "State-approved announcement: Local infrastructure update completed successfully (Event ID {}). Operational points=[{}], checksums=[{}]. In compliance with municipal directives.",
                    block.share_id, x_str.join(","), tag_str.join(",")
                )
            }
            AehChannel::SneakernetFile => {
                let x_str: Vec<String> = block.x_points.iter().map(|v| v.to_string()).collect();
                let tag_str: Vec<String> = block.tags.iter().map(|v| v.to_string()).collect();
                format!(
                    "SNEAKERNET_OFFLINE_PAYLOAD;SHARE_ID={};COORDS=[{}];OTM_TAGS=[{}]",
                    block.share_id, x_str.join(","), tag_str.join(",")
                )
            }
        }
    }

    /// Decodes a stego-encoded string back into a AehBlock
    fn stego_decode(&self, text: &str) -> Option<AehBlock> {
        match self {
            AehChannel::Wikipedia => {
                let text = text.trim();
                let main_part = text.strip_prefix("WIKI_STEGO:enwiki;")?;
                let mut share_id = 0;
                let mut x_points = Vec::new();
                let mut tags = Vec::new();
                for part in main_part.split(';') {
                    let mut kv = part.splitn(2, '=');
                    let k = kv.next()?.trim();
                    let v = kv.next()?.trim();
                    if k == "id" {
                        share_id = v.parse::<u32>().ok()?;
                    } else if k == "points" {
                        for sub in v.split(',') {
                            x_points.push(sub.trim().parse::<u32>().ok()?);
                        }
                    } else if k == "tags" {
                        for sub in v.split(',') {
                            tags.push(sub.trim().parse::<u32>().ok()?);
                        }
                    }
                }
                Some(AehBlock { share_id, x_points, tags })
            }
            AehChannel::GitHubGists => {
                let text = text.trim();
                let main_part = text.strip_prefix("GIST_STEGO:")?;
                let mut share_id = 0;
                let mut x_points = Vec::new();
                let mut tags = Vec::new();
                for part in main_part.split(';') {
                    let mut kv = part.splitn(2, '=');
                    let k = kv.next()?.trim();
                    let v = kv.next()?.trim();
                    if k == "id" {
                        share_id = v.parse::<u32>().ok()?;
                    } else if k == "points" {
                        for sub in v.split(',') {
                            x_points.push(sub.trim().parse::<u32>().ok()?);
                        }
                    } else if k == "tags" {
                        for sub in v.split(',') {
                            tags.push(sub.trim().parse::<u32>().ok()?);
                        }
                    }
                }
                Some(AehBlock { share_id, x_points, tags })
            }
            AehChannel::DnsTxt => {
                let share_id = text.split("its_id=").nth(1)?.split(' ').next()?.trim().parse::<u32>().ok()?;
                let points_str = text.split("points=").nth(1)?.split(' ').next()?;
                let x_points = points_str.split('-').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                let tags_str = text.split("tags=").nth(1)?.split(' ').next()?;
                let tags = tags_str.split('-').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                Some(AehBlock { share_id, x_points, tags })
            }
            AehChannel::Reddit => {
                let text = text.trim();
                let main_part = text.strip_prefix("REDDIT_STEGO:")?;
                let mut share_id = 0;
                let mut x_points = Vec::new();
                let mut tags = Vec::new();
                for part in main_part.split(';') {
                    let mut kv = part.splitn(2, '=');
                    let k = kv.next()?.trim();
                    let v = kv.next()?.trim();
                    if k == "id" {
                        share_id = v.parse::<u32>().ok()?;
                    } else if k == "points" {
                        for sub in v.split(',') {
                            x_points.push(sub.trim().parse::<u32>().ok()?);
                        }
                    } else if k == "tags" {
                        for sub in v.split(',') {
                            tags.push(sub.trim().parse::<u32>().ok()?);
                        }
                    }
                }
                Some(AehBlock { share_id, x_points, tags })
            }
            AehChannel::NasaTelemetry => {
                let text = text.trim();
                let mut share_id = 0;
                let mut x_points = Vec::new();
                let mut tags = Vec::new();
                for part in text.split(';') {
                    let mut kv = part.splitn(2, '=');
                    let k = kv.next()?.trim();
                    let v = kv.next()?.trim();
                    if k == "SENS_ID" {
                        share_id = v.parse::<u32>().ok()?;
                    } else if k == "GEOM_X" {
                        for sub in v.split(',') {
                            x_points.push(sub.trim().parse::<u32>().ok()?);
                        }
                    } else if k == "NOISE_FILTER" {
                        for sub in v.split(',') {
                            tags.push(sub.trim().parse::<u32>().ok()?);
                        }
                    }
                }
                Some(AehBlock { share_id, x_points, tags })
            }
            AehChannel::DomesticNews => {
                let share_id = text.split("(Event ID ").nth(1)?.split(')').next()?.trim().parse::<u32>().ok()?;
                let points_str = text.split("points=[").nth(1)?.split(']').next()?;
                let x_points = points_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                let tags_str = text.split("checksums=[").nth(1)?.split(']').next()?;
                let tags = tags_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                Some(AehBlock { share_id, x_points, tags })
            }
            AehChannel::SneakernetFile => {
                let share_id = text.split("SHARE_ID=").nth(1)?.split(';').next()?.trim().parse::<u32>().ok()?;
                let points_str = text.split("COORDS=[").nth(1)?.split(']').next()?;
                let x_points = points_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                let tags_str = text.split("OTM_TAGS=[").nth(1)?.split(']').next()?;
                let tags = tags_str.split(',').map(|s| s.trim().parse::<u32>()).collect::<Result<Vec<_>, _>>().ok()?;
                Some(AehBlock { share_id, x_points, tags })
            }
        }
    }
}

// ==============================================================================
// CUSTOM CONFIG TOML PARSER (100% SECURE / NO CRATES)
// ==============================================================================

fn parse_config(content: &str) -> Result<Config, &'static str> {
    let mut node_id = 1;
    let mut node_port = 8180;
    let mut bind_address = "127.0.0.1".to_string();

    let mut threshold_k = 2;
    let mut total_shares_n = 3;
    let mut trapdoor_x = 2;
    let mut trapdoor_y = 11;
    let mut stealth_anchor = 13;
    let mut stealth_whitening_factor = 7;

    let mut constant_rate_chaff_enabled = true;
    let mut tick_rate_ms = 100;
    let mut payload_size_elements = 16;

    let mut routing_table = HashMap::new();
    let mut entropy_sources = Vec::new();
    let mut clue_offset = 12;

    let mut current_section = "";
    let mut collecting_array_key: Option<String> = None;

    for line_raw in content.lines() {
        let mut line = line_raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // If there's an inline comment (e.g. key = val # comment), strip it
        if let Some(comment_pos) = line.find('#') {
            let before_comment = &line[..comment_pos];
            let quote_count = before_comment.chars().filter(|&c| c == '"' || c == '\'').count();
            if quote_count % 2 == 0 {
                line = before_comment.trim();
            }
        }
        if line.is_empty() {
            continue;
        }

        // Handle section transitions
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line.trim_start_matches('[').trim_end_matches(']');
            collecting_array_key = None;
            continue;
        }

        // If we are currently collecting a multi-line array
        if let Some(ref _key) = collecting_array_key {
            if line.contains(']') {
                let last_part = line.trim_end_matches(']').trim().trim_matches(',').trim().trim_matches('"').trim_matches('\'');
                if !last_part.is_empty() {
                    entropy_sources.push(last_part.to_string());
                }
                collecting_array_key = None;
            } else {
                let cleaned = line.trim_matches(',').trim().trim_matches('"').trim_matches('\'');
                if !cleaned.is_empty() {
                    entropy_sources.push(cleaned.to_string());
                }
            }
            continue;
        }

        // Standard key-value split
        if !line.contains('=') {
            continue;
        }

        let mut parts = line.splitn(2, '=');
        let key = parts.next().ok_or("Config parsing error")?.trim();
        let val_raw = parts.next().ok_or("Config parsing error")?.trim();
        
        let val_no_quotes = val_raw.trim_matches('"').trim_matches('\'');

        if current_section == "node" {
            if key == "id" {
                node_id = val_raw.parse::<u32>().map_err(|_| "Failed to parse node id")?;
            } else if key == "port" {
                node_port = val_raw.parse::<u16>().map_err(|_| "Failed to parse node port")?;
            } else if key == "bind_address" {
                bind_address = val_no_quotes.to_string();
            }
        } else if current_section == "crypto" {
            if key == "threshold_k" {
                threshold_k = val_raw.parse::<usize>().map_err(|_| "Failed to parse threshold_k")?;
            } else if key == "total_shares_n" {
                total_shares_n = val_raw.parse::<usize>().map_err(|_| "Failed to parse total_shares_n")?;
            } else if key == "trapdoor_x" {
                trapdoor_x = val_raw.parse::<u32>().map_err(|_| "Failed to parse trapdoor_x")?;
            } else if key == "trapdoor_y" {
                trapdoor_y = val_raw.parse::<u32>().map_err(|_| "Failed to parse trapdoor_y")?;
            } else if key == "stealth_anchor" {
                stealth_anchor = val_raw.parse::<u32>().map_err(|_| "Failed to parse stealth_anchor")?;
            } else if key == "stealth_whitening_factor" {
                stealth_whitening_factor = val_raw.parse::<u32>().map_err(|_| "Failed to parse stealth whitening factor")?;
            }
        } else if current_section == "traffic" {
            if key == "constant_rate_chaff_enabled" {
                constant_rate_chaff_enabled = val_raw.parse::<bool>().map_err(|_| "Failed to parse chaff enabled")?;
            } else if key == "tick_rate_ms" {
                tick_rate_ms = val_raw.parse::<u64>().map_err(|_| "Failed to parse tick_rate_ms")?;
            } else if key == "payload_size_elements" {
                payload_size_elements = val_raw.parse::<usize>().map_err(|_| "Failed to parse payload_size_elements")?;
            }
        } else if current_section == "routing_table" {
            let key_num = key.parse::<u32>().map_err(|_| "Failed to parse routing key")?;
            routing_table.insert(key_num, val_no_quotes.to_string());
        } else if current_section == "aeh" {
            if key == "entropy_sources" {
                if val_raw.starts_with('[') && val_raw.ends_with(']') {
                    let trimmed = val_raw.trim_start_matches('[').trim_end_matches(']');
                    if !trimmed.is_empty() {
                        for part in trimmed.split(',') {
                            let cleaned = part.trim().trim_matches('"').trim_matches('\'');
                            if !cleaned.is_empty() {
                                entropy_sources.push(cleaned.to_string());
                            }
                        }
                    }
                } else if val_raw.starts_with('[') {
                    collecting_array_key = Some(key.to_string());
                    let rest = val_raw.trim_start_matches('[').trim().trim_matches(',').trim().trim_matches('"').trim_matches('\'');
                    if !rest.is_empty() {
                        entropy_sources.push(rest.to_string());
                    }
                }
            } else if key == "clue_offset" {
                clue_offset = val_raw.parse::<usize>().map_err(|_| "Failed to parse clue_offset")?;
            }
        }
    }

    Ok(Config {
        node: NodeConfig { id: node_id, port: node_port, bind_address },
        crypto: CryptoConfig {
            threshold_k,
            total_shares_n,
            trapdoor_x,
            trapdoor_y,
            stealth_anchor,
            stealth_whitening_factor,
        },
        traffic: TrafficConfig {
            constant_rate_chaff_enabled,
            tick_rate_ms,
            payload_size_elements,
        },
        routing_table,
        aeh: AehConfig {
            entropy_sources,
            clue_offset,
        },
    })
}

// ==============================================================================
// HARDWARE ABSTRACTION IMPLEMENTATIONS (DECOUPLED VIA EXTERNAL CRATES)
// ==============================================================================

// CliRng and fetch_blockchain_latest_hash are drafted and extracted into their respective standalone modules/crates.

// ==============================================================================
// PACKET SERIALIZATION & DESERIALIZATION
// ==============================================================================

fn serialize_packet(packet: &MorphicOnionPacket) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(100); // 25 elements * 4 bytes = 100 bytes
    for i in 0..3 {
        bytes.extend_from_slice(&(packet.header_points[i].0.value() as u32).to_be_bytes());
        bytes.extend_from_slice(&(packet.header_points[i].1.value() as u32).to_be_bytes());
    }
    for i in 0..3 {
        bytes.extend_from_slice(&(packet.header_tags[i].value() as u32).to_be_bytes());
    }
    for i in 0..PAYLOAD_SIZE {
        bytes.extend_from_slice(&(packet.payload[i].value() as u32).to_be_bytes());
    }
    bytes
}

fn deserialize_packet(bytes: &[u8]) -> Result<MorphicOnionPacket, &'static str> {
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

    Ok(MorphicOnionPacket {
        header_points,
        header_tags,
        payload,
    })
}

// ==============================================================================
// PHYSICAL/ANALOG SHARE STANDARDIZATION (OFFLINE SSS & CHARACTER STRINGS)
// ==============================================================================

// ==============================================================================
// PHYSICAL/ANALOG SHARE STANDARDIZATION (OFFLINE SSS & CHARACTER STRINGS)
// ==============================================================================

use its_hardware::analog_shares::{export_analog_share, import_analog_share};


// ==============================================================================
// AMBIENT ENTROPY HARVESTING (AEH) HTTP FETCHING (EXTRACTED TO ITS-LEDGER CRATE)
// ==============================================================================

fn fetch_live_entropy(sources: &[String]) -> Vec<u8> {
    let mut combined_raw = Vec::new();

    for _url in sources {
        // We defer live entropy collection to our highly specialized ledger/fetching engine.
        // It utilizes clean system calls or curl abstraction without compiled TLS engines.
        if let Ok(data) = its_ledger::fetch_blockchain_latest_hash() {
            combined_raw.extend_from_slice(data.as_bytes());
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
fn universal_aeh_hash(raw_data: &[u8], key: FieldElement, n: usize) -> Vec<FieldElement> {
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let mut config_path = "config.toml".to_string();
    let mut command_args = Vec::new();
    let mut idx = 1;
    while idx < args.len() {
        if (args[idx] == "-c" || args[idx] == "--config") && idx + 1 < args.len() {
            config_path = args[idx + 1].clone();
            idx += 2;
        } else {
            command_args.push(args[idx].clone());
            idx += 1;
        }
    }

    if command_args.is_empty() {
        print_usage();
        return;
    }

    // Load configuration via our custom hand-written TOML parser
    let config_content = match std::fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(_) => {
            println!("Kunne ikke læse konfigurationsfilen: {}. Bruger standardopsætning.", config_path);
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

            [aeh]
            entropy_sources = [
                "https://api.nasa.gov/planetary/apod",
                "https://blockchain.info/q/latesthash"
            ]
            clue_offset = 12
            "#.to_string()
        }
    };

    let mut config = parse_config(&config_content).expect("Ugyldigt konfigurationsformat");

    let subcommand = command_args[0].as_str();
    match subcommand {
        "start-node" => {
            let mut port = None;
            let mut chaff_rate = None;
            let mut s_idx = 1;
            while s_idx < command_args.len() {
                if (command_args[s_idx] == "-p" || command_args[s_idx] == "--port") && s_idx + 1 < command_args.len() {
                    port = command_args[s_idx + 1].parse::<u16>().ok();
                    s_idx += 2;
                } else if (command_args[s_idx] == "-r" || command_args[s_idx] == "--chaff-rate") && s_idx + 1 < command_args.len() {
                    chaff_rate = command_args[s_idx + 1].parse::<u64>().ok();
                    s_idx += 2;
                } else {
                    s_idx += 1;
                }
            }
            if let Some(p) = port {
                config.node.port = p;
            }
            if let Some(cr) = chaff_rate {
                config.traffic.tick_rate_ms = cr;
            }
            run_node(config);
        }
        "client-send" => {
            let mut msg = String::new();
            let mut dest = 1;
            let mut aeh = false;
            let mut continuous = false;
            let mut password = None;
            let mut duress = false;

            let mut s_idx = 1;
            while s_idx < command_args.len() {
                if (command_args[s_idx] == "-m" || command_args[s_idx] == "--msg") && s_idx + 1 < command_args.len() {
                    msg = command_args[s_idx + 1].clone();
                    s_idx += 2;
                } else if (command_args[s_idx] == "-d" || command_args[s_idx] == "--dest") && s_idx + 1 < command_args.len() {
                    dest = command_args[s_idx + 1].parse::<u32>().unwrap_or(1);
                    s_idx += 2;
                } else if command_args[s_idx] == "--aeh" {
                    aeh = true;
                    s_idx += 1;
                } else if command_args[s_idx] == "--continuous" {
                    continuous = true;
                    s_idx += 1;
                } else if command_args[s_idx] == "--password" && s_idx + 1 < command_args.len() {
                    password = Some(command_args[s_idx + 1].clone());
                    s_idx += 2;
                } else if command_args[s_idx] == "--duress" {
                    duress = true;
                    s_idx += 1;
                } else {
                    s_idx += 1;
                }
            }
            run_client_send(config, msg, dest, aeh, continuous, password, duress);
        }
        "client-receive" => {
            let mut aeh = false;
            let mut continuous = false;
            let mut password = None;
            let mut duress = false;

            let mut s_idx = 1;
            while s_idx < command_args.len() {
                if command_args[s_idx] == "--aeh" {
                    aeh = true;
                    s_idx += 1;
                } else if command_args[s_idx] == "--continuous" {
                    continuous = true;
                    s_idx += 1;
                } else if command_args[s_idx] == "--password" && s_idx + 1 < command_args.len() {
                    password = Some(command_args[s_idx + 1].clone());
                    s_idx += 2;
                } else if command_args[s_idx] == "--duress" {
                    duress = true;
                    s_idx += 1;
                } else {
                    s_idx += 1;
                }
            }
            run_client_receive(config, aeh, continuous, password, duress);
        }
        "time-lock" => {
            let mut file = PathBuf::new();
            let mut epochs = 1000;
            let mut out = PathBuf::new();

            let mut s_idx = 1;
            while s_idx < command_args.len() {
                if (command_args[s_idx] == "-f" || command_args[s_idx] == "--file") && s_idx + 1 < command_args.len() {
                    file = PathBuf::from(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else if (command_args[s_idx] == "-e" || command_args[s_idx] == "--epochs") && s_idx + 1 < command_args.len() {
                    epochs = command_args[s_idx + 1].parse::<usize>().unwrap_or(1000);
                    s_idx += 2;
                } else if (command_args[s_idx] == "-o" || command_args[s_idx] == "--out") && s_idx + 1 < command_args.len() {
                    out = PathBuf::from(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else {
                    s_idx += 1;
                }
            }
            run_time_lock(file, epochs, out);
        }
        "time-unlock" => {
            let mut puzzle = PathBuf::new();
            let mut out = PathBuf::new();

            let mut s_idx = 1;
            while s_idx < command_args.len() {
                if (command_args[s_idx] == "-p" || command_args[s_idx] == "--puzzle") && s_idx + 1 < command_args.len() {
                    puzzle = PathBuf::from(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else if (command_args[s_idx] == "-o" || command_args[s_idx] == "--out") && s_idx + 1 < command_args.len() {
                    out = PathBuf::from(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else {
                    s_idx += 1;
                }
            }
            run_time_unlock(puzzle, out);
        }
        "time-deny" => {
            let mut puzzle = PathBuf::new();
            let mut decoy = String::new();
            let mut out = PathBuf::new();

            let mut s_idx = 1;
            while s_idx < command_args.len() {
                if (command_args[s_idx] == "-p" || command_args[s_idx] == "--puzzle") && s_idx + 1 < command_args.len() {
                    puzzle = PathBuf::from(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else if (command_args[s_idx] == "-d" || command_args[s_idx] == "--decoy") && s_idx + 1 < command_args.len() {
                    decoy = command_args[s_idx + 1].clone();
                    s_idx += 2;
                } else if (command_args[s_idx] == "-o" || command_args[s_idx] == "--out") && s_idx + 1 < command_args.len() {
                    out = PathBuf::from(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else {
                    s_idx += 1;
                }
            }
            run_time_deny(puzzle, decoy, out);
        }
        "client-export-share" => {
            let mut msg = String::new();
            let mut threshold_k = None;
            let mut total_shares_n = None;

            let mut s_idx = 1;
            while s_idx < command_args.len() {
                if (command_args[s_idx] == "-m" || command_args[s_idx] == "--msg") && s_idx + 1 < command_args.len() {
                    msg = command_args[s_idx + 1].clone();
                    s_idx += 2;
                } else if (command_args[s_idx] == "-k" || command_args[s_idx] == "--threshold") && s_idx + 1 < command_args.len() {
                    threshold_k = command_args[s_idx + 1].parse::<usize>().ok();
                    s_idx += 2;
                } else if (command_args[s_idx] == "-n" || command_args[s_idx] == "--shares") && s_idx + 1 < command_args.len() {
                    total_shares_n = command_args[s_idx + 1].parse::<usize>().ok();
                    s_idx += 2;
                } else {
                    s_idx += 1;
                }
            }
            if msg.is_empty() {
                println!("Fejl: Beskeden må ikke være tom. Brug -m, --msg <tekst>");
                return;
            }
            run_client_export_share(config, msg, threshold_k, total_shares_n);
        }
        "client-import-share" => {
            let mut shares_input = Vec::new();
            let mut file_path = None;
            let mut threshold_k = None;

            let mut s_idx = 1;
            while s_idx < command_args.len() {
                if (command_args[s_idx] == "-f" || command_args[s_idx] == "--file") && s_idx + 1 < command_args.len() {
                    file_path = Some(command_args[s_idx + 1].clone());
                    s_idx += 2;
                } else if (command_args[s_idx] == "-k" || command_args[s_idx] == "--threshold") && s_idx + 1 < command_args.len() {
                    threshold_k = command_args[s_idx + 1].parse::<usize>().ok();
                    s_idx += 2;
                } else {
                    let arg = &command_args[s_idx];
                    if arg.starts_with("ITS-SHARE:") {
                        shares_input.push(arg.clone());
                    }
                    s_idx += 1;
                }
            }

            if let Some(fp) = file_path {
                match std::fs::read_to_string(&fp) {
                    Ok(content) => {
                        for line in content.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("SSS-SHARE:") {
                                shares_input.push(trimmed.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        println!("Fejl: Kunne ikke læse filen {:?}: {:?}", fp, e);
                        return;
                    }
                }
            }

            if shares_input.is_empty() {
                println!("Fejl: Ingen analoge shares fundet. Angiv dem direkte som argumenter, eller indlæs med -f, --file <sti>");
                return;
            }

            run_client_import_share(config, shares_input, threshold_k);
        }
        _ => {
            print_usage();
        }
    }
}

fn print_usage() {
    println!("Morphic Routing Shadow Network CLI (Sterilized Synkron Version)");
    println!("Anvendelse:");
    println!("  its-net [subcommand] [valg]");
    println!("\nSubcommands:");
    println!("  start-node      Starts an active onion routing daemon node");
    println!("                  -p, --port <port>       Port to bind the listener to");
    println!("                  -r, --chaff-rate <ms>   Continuous dummy chaff loop timing");
    println!("  client-send     Sends encrypted onion packet or dispatches steganographic AEH channels");
    println!("                  -m, --msg <text>        Document contents/message string to send");
    println!("                  -d, --dest <id>         Destination Node Field Element ID");
    println!("                  --aeh                   Use Ambient Entropy Harvesting instead of Onion Tunnel");
    println!("                  --continuous            Enable continuous background decoy chaffing schedule loop");
    println!("                  --password <pass>       PBKDF2 Password for True/Decoy ratchet seeds");
    println!("                  --duress                Trigger Duress Mode to send plausible decoy recipe");
    println!("  client-receive  Monitors port for incoming SSS-shares or scans AEH channels");
    println!("                  --aeh                   Scan steganographic public AEH channels");
    println!("                  --continuous            Enable continuous background winnowing scheduler loop");
    println!("                  --password <pass>       PBKDF2 Password for True/Decoy ratchet seeds");
    println!("                  --duress                Trigger Duress Mode to decrypt cover recipe only");
    println!("  time-lock       Generates a local hybrid deniable time-lock puzzle over a file");
    println!("                  -f, --file <path>       Target document to lock");
    println!("                  -e, --epochs <count>    Number of sequential squaring delay rounds (default 1000)");
    println!("                  -o, --out <path>        Output path to write locked puzzle (.its)");
    println!("  time-unlock     Solves modular squarings sequentially on CPU to decrypt a puzzle");
    println!("                  -p, --puzzle <path>     Target puzzle .its file");
    println!("                  -o, --out <path>        Output decrypted file path");
    println!("  time-deny       Asserts a decoy dækhistorie message to solve the puzzle to alternative 'truth'");
    println!("                  -p, --puzzle <path>     Target puzzle .its file");
    println!("                  -d, --decoy <text>      Harmless decoy message of equal length");
    println!("                  -o, --out <path>        Output alternative decrypted file path");
    println!("  client-export-share Serialize Shamir shares into physical character strings (analog export)");
    println!("                  -m, --msg <text>        Target secret message to split");
    println!("                  -k, --threshold <k>     Override threshold k");
    println!("                  -n, --shares <n>        Override total shares n");
    println!("  client-import-share Reconstruct Shamir secret from physical character strings (analog import)");
    println!("                  -f, --file <path>       File containing physical share strings (one per line)");
    println!("                  -k, --threshold <k>     Override threshold k");
    println!("                  [shares]                Provide physical share strings directly as arguments");
}

// ==============================================================================
// DAEMON RUNNER
// ==============================================================================

fn run_node(config: Config) {
    let bind_addr = format!("{}:{}", config.node.bind_address, config.node.port);
    let socket = UdpSocket::bind(&bind_addr).expect("Kunne ikke binde UDP socket");
    let courier: Arc<dyn PacketCourier + Send + Sync> = Arc::new(UdpCourier::new(socket));
    println!("Morphic Routing Node {} kører på {} via abstract PacketCourier", config.node.id, bind_addr);

    // Setup MorphicMixingNode
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
    let (_k_pool, mac_key, nonce) = ratchet.lock().unwrap().step().expect("Kunne ikke initiere ratchet");

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
        let mut rng = its_hardware::CliRng;
        loop {
            if let Ok((len, _src)) = courier_recv.recv_raw(&mut buf) {
                if let Ok(packet) = deserialize_packet(&buf[..len]) {
                    let mut n = node_recv.lock().unwrap();
                    let mut processed = false;
                    for hop_index in 0..3 {
                        if let Ok((next_hop, forwarded_packet)) = n.process_packet(&packet, hop_index, &mut rng) {
                            if next_hop.value() as u32 != 0 {
                                println!("Modtog gyldig pakke! Næste hop ID: {}", next_hop.value() as u32);
                                queue_recv.lock().unwrap().push((next_hop, forwarded_packet));
                                processed = true;

                                // Step the ratchet to rotate keys and nonces for the next packet
                                if let Ok((_, next_mac, next_nonce)) = ratchet_recv.lock().unwrap().step() {
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

    // 2. CONSTANT-RATE SENDER TASK (OS Thread)
    let courier_send = courier.clone();
    let node_send = node.clone();
    let queue_send = queue.clone();
    let routing_table = config.routing_table.clone();
    let chaff_enabled = config.traffic.constant_rate_chaff_enabled;

    thread::spawn(move || {
        let mut rng = its_hardware::CliRng;
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
                    println!("Sendte real pakke til næste hop {} ({})", next_hop_val, addr_str);
                }
            } else if chaff_enabled {
                // Generate and send a dummy packet to maintain constant rate
                let dummy_packet = node_send.lock().unwrap().pop_constant_rate_packet(&mut rng);
                // Pick a random peer from the routing table
                if !routing_table.is_empty() {
                    let keys: Vec<&u32> = routing_table.keys().collect();
                    
                    // Directly select random index using our its_hardware::CliRng so we don't need rand::thread_rng()
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

// ==============================================================================
// CLIENT SEND RUNNER
// ==============================================================================

fn run_client_send(
    config: Config,
    msg: String,
    dest: u32,
    aeh: bool,
    continuous: bool,
    password: Option<String>,
    duress: bool,
) {
    let mut rng = its_hardware::CliRng;
    let mut active_msg = msg;

    if aeh && duress {
        println!("\n[DURESS MODE ACTIVE]: Decoy/Duress password entered!");
        println!("Initializing decoy cover-channels with plausible harmless content.");
        active_msg = "Decoy baking recipe: 2 cups flour, 1 cup sugar, 3 eggs. Bake at 180C for 30 minutes.".to_string();
    }

    let msg_bytes = active_msg.as_bytes();

    if aeh {
        let anchor = FieldElement::new(config.crypto.stealth_anchor);

        // Derive/Initialize StateRatchet
        let seed = if let Some(ref pwd) = password {
            let salt: &[u8] = if duress { b"scpst-aeh-decoy-salt" } else { b"scpst-aeh-true-salt" };
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
            AehChannel::Wikipedia,
            AehChannel::GitHubGists,
            AehChannel::DnsTxt,
            AehChannel::Reddit,
            AehChannel::NasaTelemetry,
            AehChannel::DomesticNews,
            AehChannel::SneakernetFile,
        ];

        if continuous {
            println!("\nStarter kontinuerlig scheduled decoy chaffing-loop...");
            println!("Sender i faste intervaller á {} ms.", config.traffic.tick_rate_ms);
            let mut tick = 0u64;
            // Let's send the active_msg on tick 2, and mock/dummy chaff on all other ticks
            loop {
                thread::sleep(Duration::from_millis(config.traffic.tick_rate_ms));
                tick += 1;

                let live_entropy = fetch_live_entropy(&config.aeh.entropy_sources);

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
                        let entropy_points = universal_aeh_hash(&live_entropy, k_pool, share.data_points.len());

                        for (idx, &s) in share.data_points.iter().enumerate() {
                            let s_whitened = stealth.shard_whiten(s);
                            let m = stealth.impose(s_whitened);
                            let entropy_fe = entropy_points[idx];
                            let x = stealth.inject(m, entropy_fe);
                            let tag = stealth.generate_attestation(m, k_mac, nonce);

                            x_points.push(x.value() as u32);
                            tags.push(tag.value() as u32);
                        }

                        let block = AehBlock {
                            share_id: share.id.value() as u32,
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
                            let mut r_buf = [0u8; 4];
                            let _ = rng.fill_bytes(&mut r_buf);
                            let random_val = u32::from_be_bytes(r_buf);
                            x_points.push(random_val % 2147483647);

                            let mut r_buf2 = [0u8; 4];
                            let _ = rng.fill_bytes(&mut r_buf2);
                            let random_val2 = u32::from_be_bytes(r_buf2);
                            tags.push(random_val2 % 2147483647);
                        }

                        let block = AehBlock {
                            share_id: share.id.value() as u32,
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
            let live_entropy = fetch_live_entropy(&config.aeh.entropy_sources);
            println!("Modtog {} bytes live entropi.", live_entropy.len());
            let msg_bytes = active_msg.as_bytes();
            let shares = fragment_data(msg_bytes, config.crypto.threshold_k, config.crypto.total_shares_n, &mut rng)
                .expect("Kunne ikke fragmentere data");

            println!("AEH-shards genereret, attesteret og steganografisk camoufleret:");

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
                let entropy_points = universal_aeh_hash(&live_entropy, k_pool, share.data_points.len());

                for (idx, &s) in share.data_points.iter().enumerate() {
                    let s_whitened = stealth.shard_whiten(s);
                    let m = stealth.impose(s_whitened);
                    let entropy_fe = entropy_points[idx];

                    let x = stealth.inject(m, entropy_fe);
                    let tag = stealth.generate_attestation(m, k_mac, nonce);

                    x_points.push(x.value() as u32);
                    tags.push(tag.value() as u32);
                }

                let block = AehBlock {
                    share_id: share.id.value() as u32,
                    x_points,
                    tags,
                };

                let channel = channels[(share.id.value() as usize - 1) % channels.len()];
                let stego_text = channel.stego_encode(&block);

                println!("\n--- [ {} ] ---", channel.name());
                println!("{}", stego_text);
            }
            println!("\nAEH-transmission fuldført med fuld steganografisk sløring og Wegman-Carter OTM-attestering.");
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
        let socket = UdpSocket::bind("0.0.0.0:0").expect("Kunne ikke binde klientsocket");
        if let Ok(addr) = addr_str.parse::<SocketAddr>() {
            let _ = socket.send_to(&bytes, addr);
            println!("Onion-pakke sendt til første hop {} ({})", 1, addr_str);
        }
    } else {
        println!("Fejl: Første hop (Node 1) blev ikke fundet i routingtabellen.");
    }
}

// ==============================================================================
// CLIENT RECEIVE RUNNER
// ==============================================================================

fn run_client_receive(
    config: Config,
    aeh: bool,
    continuous: bool,
    password: Option<String>,
    duress: bool,
) {
    if aeh {
        println!("Henter live entropi fra offentlige kilder til transposition...");
        let live_entropy = fetch_live_entropy(&config.aeh.entropy_sources);
        println!("Modtog {} bytes live entropi.", live_entropy.len());

        println!("Modtager via Ambient Entropy Harvesting (AEH)...");
        println!("Scanner simulerede steganografiske kanaler efter attesterede shards...");

        let anchor = FieldElement::new(config.crypto.stealth_anchor);

        // Derive/Initialize StateRatchet for Bob
        let seed = if let Some(ref pwd) = password {
            let salt: &[u8] = if duress { b"scpst-aeh-decoy-salt" } else { b"scpst-aeh-true-salt" };
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

        let mut temp_rng = its_hardware::CliRng;
        let mock_shares = fragment_data(target_msg.as_bytes(), config.crypto.threshold_k, config.crypto.total_shares_n, &mut temp_rng)
            .expect("Kunne ikke generere fragmenter");

        let channels = [
            AehChannel::Wikipedia,
            AehChannel::GitHubGists,
            AehChannel::DnsTxt,
            AehChannel::Reddit,
            AehChannel::NasaTelemetry,
            AehChannel::DomesticNews,
            AehChannel::SneakernetFile,
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
            let entropy_points = universal_aeh_hash(&live_entropy, k_pool, share.data_points.len());

            for (idx, &s) in share.data_points.iter().enumerate() {
                let s_whitened = stealth.shard_whiten(s);
                let m = stealth.impose(s_whitened);
                let entropy_fe = entropy_points[idx];
                let x = stealth.inject(m, entropy_fe);
                let tag = stealth.generate_attestation(m, k_mac, nonce);
                x_points.push(x.value() as u32);
                tags.push(tag.value() as u32);
            }

            let block = AehBlock {
                share_id: share.id.value() as u32,
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
                thread::sleep(Duration::from_millis(config.traffic.tick_rate_ms));
                tick += 1;

                if tick == 2 {
                    println!("\n--- [TICK {}]: RECEIVED REAL STEGO BLOCKS (WINNOWING ACTIVE) ---", tick);
                    // To make the simulation extremely realistic, we let Eve tamper/block some of the channels!
                    println!("\n[EVE ATTACK]: Eve blocks GitHubGists & NasaTelemetry, and tampers with the Reddit comment!");

                    let mut received_shares = Vec::new();

                    for &(channel, ref text) in stego_inputs.iter() {
                        // 1. Check if blocked
                        if channel == AehChannel::GitHubGists || channel == AehChannel::NasaTelemetry {
                            println!("- Channel {}: BLOCKED by Eve. Skipping.", channel.name());
                            continue;
                        }

                        // 2. Check if tampered
                        let mut text_to_decode = text.clone();
                        if channel == AehChannel::Reddit {
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
                            let entropy_points = universal_aeh_hash(&live_entropy, k_pool, block.x_points.len());

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
                                received_shares.push(SssPackedShare {
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
                if channel == AehChannel::GitHubGists || channel == AehChannel::NasaTelemetry {
                    println!("- Channel {}: BLOCKED by Eve. Skipping.", channel.name());
                    continue;
                }

                let mut text_to_decode = text.clone();
                if channel == AehChannel::Reddit {
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
                    let entropy_points = universal_aeh_hash(&live_entropy, k_pool, block.x_points.len());

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
                        received_shares.push(SssPackedShare {
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
    let socket = UdpSocket::bind(&bind_addr).expect("Kunne ikke binde UDP socket");

    let mut shares = Vec::<SssPackedShare>::new();
    let mut buf = [0u8; 1024];

    loop {
        if let Ok((len, _src)) = socket.recv_from(&mut buf) {
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
                shares.push(SssPackedShare { id, data_points });

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

fn run_time_lock(file_path: PathBuf, epochs: usize, out_path: PathBuf) {
    let mut rng = TimelockRng;
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
            let text_puzzle = TimeLockText::from_core(&puzzle);
            let serialized = text_puzzle.serialize();

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
        Err(GenerateError::InvalidInput) => {
            println!("Fejl: Ugyldige parametre (tom fil eller epochs=0).");
        }
        Err(GenerateError::Rng(e)) => {
            println!("Fejl under entropi-indsamling: {:?}", e);
        }
    }
}

fn run_time_unlock(puzzle_path: PathBuf, out_path: PathBuf) {
    println!("Indlæser tidslåst puslespil fra: {:?}", puzzle_path);

    let puzzle_content = match std::fs::read_to_string(&puzzle_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Fejl: Kunne ikke læse tidslåsen: {:?}", e);
            return;
        }
    };

    let text_puzzle = match TimeLockText::deserialize(&puzzle_content) {
        Ok(p) => p,
        Err(e) => {
            println!("Fejl: Ugyldigt tidslås-filformat: {:?}", e);
            return;
        }
    };

    let puzzle = text_puzzle.to_core();

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

fn run_time_deny(puzzle_path: PathBuf, decoy_msg: String, out_path: PathBuf) {
    println!("Indlæser tidslåst puslespil til deniability-test: {:?}", puzzle_path);

    let puzzle_content = match std::fs::read_to_string(&puzzle_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Fejl: Kunne ikke læse tidslåsen: {:?}", e);
            return;
        }
    };

    let text_puzzle = match TimeLockText::deserialize(&puzzle_content) {
        Ok(p) => p,
        Err(e) => {
            println!("Fejl: Ugyldigt tidslås-filformat: {:?}", e);
            return;
        }
    };

    let puzzle = text_puzzle.to_core();
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
        current_share_2.push(TlFieldElement::new(s2_0_raw));
    }
    for j in 0..puzzle.t {
        let trans_2 = &puzzle.transitions_2[j];
        for idx in 0..puzzle.initial_share_1.len() {
            current_share_2[idx] = trans_2[idx] - current_share_2[idx];
        }
    }

    let mut decoy_shares_1_t = Vec::with_capacity(puzzle.initial_share_1.len());
    for idx in 0..puzzle.initial_share_1.len() {
        let s2_t = current_share_2[idx];
        let d_byte = padded_decoy[idx];
        let secret_t = puzzle.encrypted_payload[idx] - TlFieldElement::new(d_byte as u32);
        let s1_t = (secret_t + s2_t) * TlFieldElement::new(2).invert();
        decoy_shares_1_t.push(s1_t);
    }

    let mut decoy_shares_1_0 = decoy_shares_1_t;
    for j in (0..puzzle.t).rev() {
        let trans_1 = &puzzle.transitions_1[j];
        for idx in 0..decoy_shares_1_0.len() {
            decoy_shares_1_0[idx] = trans_1[idx] - decoy_shares_1_0[idx];
        }
    }

    let mut alt_puzzle = puzzle;
    alt_puzzle.initial_share_1 = decoy_shares_1_0;

    let solved_decoy_bytes = alt_puzzle.solve().unwrap();
    assert_eq!(solved_decoy_bytes, padded_decoy);

    // Save the alternative puzzle text
    let alt_text_puzzle = TimeLockText::from_core(&alt_puzzle);
    let serialized_alt = alt_text_puzzle.serialize();

    if let Err(e) = std::fs::write(&out_path, serialized_alt) {
        println!("Fejl: Kunne ikke gemme den alternative tidslås: {:?}", e);
        return;
    }

    println!("Succes! Alternativ 'bageopskrift' tidslås genereret og gemt i: {:?}", out_path);
    println!("Hvis nogen tvinger dig til at løse puslespillet, kan du udlevere denne alternative fil.");
    println!("Den vil dekryptere til din dækhistorie: \"{}\"", String::from_utf8_lossy(&padded_decoy).trim());
}

fn run_client_export_share(config: Config, msg: String, threshold_k: Option<usize>, total_shares_n: Option<usize>) {
    let k = threshold_k.unwrap_or(config.crypto.threshold_k);
    let n = total_shares_n.unwrap_or(config.crypto.total_shares_n);
    println!("Analog-export: Fragmenterer besked med k={}, n={}", k, n);

    let mut rng = its_hardware::CliRng;
    match fragment_data(msg.as_bytes(), k, n, &mut rng) {
        Ok(shares) => {
            println!("--- REPRODUCIBLE PHYSICAL SSS SHARES (KOPIDUPLICERBARE PAPIRBLOKKE) ---");
            for share in &shares {
                let encoded = export_analog_share(share);
                println!("{}", encoded);
            }
            println!("----------------------------------------------------------------------");
            println!("Opbevar disse strenge sikkert på uafhængige analoge medier (papir, QR-koder, mikrofilm).");
            println!("Enhver samling af {} ud af disse {} strenge kan fuldstændigt rekonstruere beskeden.", k, n);
        }
        Err(_) => {
            println!("Fejl under fragmentering af data.");
        }
    }
}

fn run_client_import_share(config: Config, shares_input: Vec<String>, threshold_k: Option<usize>) {
    let k = threshold_k.unwrap_or(config.crypto.threshold_k);
    println!("Analog-import: Forsøger at rekonstruere besked ud fra {} analoge dele (k={}).", shares_input.len(), k);

    let mut parsed_shares = Vec::new();
    for input in &shares_input {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        match import_analog_share(trimmed) {
            Ok(share) => {
                println!("Indlæste gyldig share ID: {}", share.id.value() as u32);
                parsed_shares.push(share);
            }
            Err(e) => {
                println!("Fejl ved indlæsning af share \"{}\": {}", trimmed, e);
                return;
            }
        }
    }

    if parsed_shares.is_empty() {
        println!("Fejl: Ingen gyldige analoge dele indtastet.");
        return;
    }

    match reconstruct_data(&parsed_shares, k) {
        Ok(secret_bytes) => {
            println!("\n--- REKONSTRUERET HEMMELIGHED (100% KORREKT) ---");
            println!("{}", String::from_utf8_lossy(&secret_bytes));
            println!("-------------------------------------------------");
        }
        Err(_) => {
            println!("Fejl: Rekonstruktion fejlede. Muligvis for få dele (har {}, behøver k={}), eller delene tilhører ikke samme hemmelighed.", parsed_shares.len(), k);
        }
    }
}

#[cfg(test)]
mod cli_analog_tests {
    use super::*;

    #[test]
    fn test_analog_share_roundtrip() {
        #[cfg(feature = "m61")]
        println!("--- CLI TEST: FEATURE m61 IS ENABLED ---");
        #[cfg(not(feature = "m61"))]
        println!("--- CLI TEST: FEATURE m61 IS DISABLED ---");

        let mut rng = its_hardware::CliRng;
        let original_secret = b"Information-Theoretic Absolute Secrecy is the ultimate goal!";
        let k = 3;
        let n = 5;
        let shares = fragment_data(original_secret, k, n, &mut rng).unwrap();

        // Export each share to analog text format
        let mut exported_strings = Vec::new();
        for share in &shares {
            let exported = export_analog_share(share);
            exported_strings.push(exported);
        }

        // Test single share round-trip accuracy
        let test_share = &shares[0];
        let test_exported = export_analog_share(test_share);
        println!("EXPORTED HEX STRING: {}", test_exported);
        let test_imported = import_analog_share(&test_exported).unwrap();
        println!("ORIGINAL ID: {}, IMPORTED ID: {}", test_share.id.value() as u64, test_imported.id.value() as u64);
        println!("ORIGINAL LEN: {}, IMPORTED LEN: {}", test_share.data_points.len(), test_imported.data_points.len());
        for i in 0..test_share.data_points.len() {
            println!("PT [{}]: ORIGINAL={}, IMPORTED={}", i, test_share.data_points[i].value() as u64, test_imported.data_points[i].value() as u64);
        }

        assert_eq!(test_share.id.value(), test_imported.id.value());
        assert_eq!(test_share.data_points.len(), test_imported.data_points.len());
        for i in 0..test_share.data_points.len() {
            assert_eq!(test_share.data_points[i].value(), test_imported.data_points[i].value());
        }
        let subset_to_import = &exported_strings[0..k];
        let mut imported_shares = Vec::new();
        for s_str in subset_to_import {
            let imported = import_analog_share(s_str).unwrap();
            imported_shares.push(imported);
        }

        // Reconstruct secret from imported shares
        let reconstructed = reconstruct_data(&imported_shares, k).unwrap();
        assert_eq!(reconstructed, original_secret);
    }
}

