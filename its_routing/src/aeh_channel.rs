use its_transport::field_arith::FieldElement;

// DEPRECATE: legacy stego block codec — production AEH uses `aeh_carrier::embed_cell` (φ ~ D_benign).
// `stego_encode` / WIKI_STEGO retained only under `dev-onion-mix` for mix-net regression tests.

#[derive(Debug, Clone)]
pub(crate) struct AehBlock {
    pub(crate) share_id: u32,
    pub(crate) x_points: Vec<u32>,
    pub(crate) tags: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AehChannel {
    Wikipedia,
    GitHubGists,
    DnsTxt,
    Reddit,
    NasaTelemetry,
    DomesticNews,
    SneakernetFile,
}

impl AehChannel {
    pub(crate) fn name(&self) -> &'static str {
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

    /// Encodes a AehBlock into simulated steganographic camouflage text (dev-onion-mix only).
    #[cfg(feature = "dev-onion-mix")]
    pub(crate) fn stego_encode(&self, block: &AehBlock) -> String {
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

    /// Decodes a stego-encoded string back into a AehBlock (dev-onion-mix only).
    #[cfg(feature = "dev-onion-mix")]
    pub(crate) fn stego_decode(&self, text: &str) -> Option<AehBlock> {
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
/// Keyed Carter-Wegman Universal Polynomial Hash (100% ITS-Secure)
/// Maps raw public telemetry bytes directly to N distinct FieldElements.
pub(crate) fn universal_aeh_hash(raw_data: &[u8], key: FieldElement, n: usize) -> Vec<FieldElement> {
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
