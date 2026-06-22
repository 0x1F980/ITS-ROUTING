use std::collections::HashMap;

// ==============================================================================
// CONFIGURATION STRUCTURES
// ==============================================================================

#[derive(Debug, Clone)]
pub struct Config {
    pub node: NodeConfig,
    pub crypto: CryptoConfig,
    pub traffic: TrafficConfig,
    pub routing_table: HashMap<u32, String>,
    pub aeh: AehConfig,
    pub fingerprint_erasure: FingerprintErasureConfig,
    pub pool: PoolConfig,
}

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub id: u32,
    pub port: u16,
    pub bind_address: String,
}

#[derive(Debug, Clone)]
pub struct CryptoConfig {
    pub threshold_k: usize,
    pub total_shares_n: usize,
    pub trapdoor_x: u32,
    pub trapdoor_y: u32,
    pub stealth_anchor: u32,
    pub stealth_whitening_factor: u32,
}

#[derive(Debug, Clone)]
pub struct TrafficConfig {
    pub constant_rate_chaff_enabled: bool,
    pub tick_rate_ms: u64,
    pub payload_size_elements: usize,
}

#[derive(Debug, Clone, Default)]
pub struct AehConfig {
    pub entropy_sources: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FingerprintErasureConfig {
    pub default_pad: String,
    pub require_otp: bool,
    pub require_chaff: bool,
    pub require_on_file_send: bool,
}

#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub transport_mode: String,
    pub epoch_interval_ms: u64,
    pub cell_size_l: usize,
    pub pool_url: String,
    pub pool_file: String,
    pub multi_pool_urls: Vec<String>,
    pub witness_pool_urls: Vec<String>,
    pub consensus_k: usize,
    pub valid_fwd_window: u64,
    pub sss_k: usize,
    pub sss_n: usize,
    pub fountain_enabled: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            transport_mode: "pool".to_string(),
            epoch_interval_ms: 100,
            cell_size_l: 4096,
            pool_url: String::new(),
            pool_file: ".its-pool".to_string(),
            multi_pool_urls: Vec::new(),
            witness_pool_urls: Vec::new(),
            consensus_k: 1,
            valid_fwd_window: 64,
            sss_k: 2,
            sss_n: 3,
            fountain_enabled: false,
        }
    }
}

impl Default for FingerprintErasureConfig {
    fn default() -> Self {
        Self {
            default_pad: String::new(),
            require_otp: true,
            require_chaff: true,
            require_on_file_send: true,
        }
    }
}

// ==============================================================================
// CUSTOM CONFIG TOML PARSER (100% SECURE / NO CRATES)
// ==============================================================================

pub fn parse_config(content: &str) -> Result<Config, &'static str> {
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

    let mut fe_default_pad = String::new();
    let mut fe_require_otp = true;
    let mut fe_require_chaff = true;
    let mut fe_require_on_file_send = true;

    let mut pool_transport_mode = "pool".to_string();
    let mut pool_epoch_interval_ms = 100u64;
    let mut pool_cell_size_l = 4096usize;
    let mut pool_url = String::new();
    let mut pool_file = ".its-pool".to_string();
    let mut multi_pool_urls = Vec::new();
    let mut witness_pool_urls = Vec::new();
    let mut pool_consensus_k = 1usize;
    let mut pool_valid_fwd_window = 64u64;
    let mut pool_sss_k = 2usize;
    let mut pool_sss_n = 3usize;
    let mut pool_fountain_enabled = false;

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
        if let Some(ref key) = collecting_array_key {
            if line.contains(']') {
                let last_part = line.trim_end_matches(']').trim().trim_matches(',').trim().trim_matches('"').trim_matches('\'');
                if !last_part.is_empty() {
                    if key == "entropy_sources" {
                        entropy_sources.push(last_part.to_string());
                    } else if key == "multi_pool_urls" {
                        multi_pool_urls.push(last_part.to_string());
                    } else if key == "witness_pool_urls" {
                        witness_pool_urls.push(last_part.to_string());
                    }
                }
                collecting_array_key = None;
            } else {
                let cleaned = line.trim_matches(',').trim().trim_matches('"').trim_matches('\'');
                if !cleaned.is_empty() {
                    if key == "entropy_sources" {
                        entropy_sources.push(cleaned.to_string());
                    } else if key == "multi_pool_urls" {
                        multi_pool_urls.push(cleaned.to_string());
                    } else if key == "witness_pool_urls" {
                        witness_pool_urls.push(cleaned.to_string());
                    }
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
            }
        } else if current_section == "fingerprint_erasure" {
            if key == "default_pad" {
                fe_default_pad = val_raw.trim_matches('"').trim_matches('\'').to_string();
            } else if key == "require_otp" {
                fe_require_otp = val_raw.parse::<bool>().map_err(|_| "Failed to parse require_otp")?;
            } else if key == "require_chaff" {
                fe_require_chaff = val_raw.parse::<bool>().map_err(|_| "Failed to parse require_chaff")?;
            } else if key == "require_on_file_send" {
                fe_require_on_file_send =
                    val_raw.parse::<bool>().map_err(|_| "Failed to parse require_on_file_send")?;
            }
        } else if current_section == "pool" {
            if key == "transport_mode" {
                pool_transport_mode = val_no_quotes.to_string();
            } else if key == "epoch_interval_ms" {
                pool_epoch_interval_ms = val_raw.parse::<u64>().map_err(|_| "Failed to parse epoch_interval_ms")?;
            } else if key == "cell_size_L" || key == "cell_size_l" {
                pool_cell_size_l = val_raw.parse::<usize>().map_err(|_| "Failed to parse cell_size_L")?;
            } else if key == "pool_url" {
                pool_url = val_no_quotes.to_string();
            } else if key == "pool_file" {
                pool_file = val_no_quotes.to_string();
            } else if key == "sss_k" {
                pool_sss_k = val_raw.parse::<usize>().map_err(|_| "Failed to parse sss_k")?;
            } else if key == "sss_n" {
                pool_sss_n = val_raw.parse::<usize>().map_err(|_| "Failed to parse sss_n")?;
            } else if key == "fountain_enabled" {
                pool_fountain_enabled = val_raw.parse::<bool>().map_err(|_| "Failed to parse fountain_enabled")?;
            } else if key == "multi_pool_urls" || key == "witness_pool_urls" {
                let target = if key == "multi_pool_urls" {
                    &mut multi_pool_urls
                } else {
                    &mut witness_pool_urls
                };
                if val_raw.starts_with('[') && val_raw.ends_with(']') {
                    let trimmed = val_raw.trim_start_matches('[').trim_end_matches(']');
                    if !trimmed.is_empty() {
                        for part in trimmed.split(',') {
                            let cleaned = part.trim().trim_matches('"').trim_matches('\'');
                            if !cleaned.is_empty() {
                                target.push(cleaned.to_string());
                            }
                        }
                    }
                } else if val_raw.starts_with('[') {
                    collecting_array_key = Some(key.to_string());
                    let rest = val_raw.trim_start_matches('[').trim().trim_matches(',').trim().trim_matches('"').trim_matches('\'');
                    if !rest.is_empty() {
                        target.push(rest.to_string());
                    }
                }
            } else if key == "consensus_k" {
                pool_consensus_k = val_raw.parse::<usize>().map_err(|_| "Failed to parse consensus_k")?;
            } else if key == "valid_fwd_window" {
                pool_valid_fwd_window = val_raw.parse::<u64>().map_err(|_| "Failed to parse valid_fwd_window")?;
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
        },
        fingerprint_erasure: FingerprintErasureConfig {
            default_pad: fe_default_pad,
            require_otp: fe_require_otp,
            require_chaff: fe_require_chaff,
            require_on_file_send: fe_require_on_file_send,
        },
        pool: PoolConfig {
            transport_mode: pool_transport_mode,
            epoch_interval_ms: pool_epoch_interval_ms,
            cell_size_l: pool_cell_size_l,
            pool_url,
            pool_file,
            multi_pool_urls,
            witness_pool_urls,
            consensus_k: pool_consensus_k,
            valid_fwd_window: pool_valid_fwd_window,
            sss_k: pool_sss_k,
            sss_n: pool_sss_n,
            fountain_enabled: pool_fountain_enabled,
        },
    })
}
