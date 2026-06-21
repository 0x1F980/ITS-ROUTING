//! Ambient Entropy Harvesting helpers — optional ledger fetch and OTM tag verify.

use std::io::{Read, Write};
use std::net::TcpStream;

use its_transport::field_arith::FieldElement;

#[cfg(feature = "otm")]
use its_otm_public_attestation::otm::verify_tag as verify_public_otm_tag;

fn http_get_body(url: &str) -> Option<Vec<u8>> {
    let stripped = url.trim().strip_prefix("http://")?;
    let (host_port, path) = match stripped.split_once('/') {
        Some((hp, rest)) => (hp, format!("/{rest}")),
        None => (stripped, "/".to_string()),
    };
    let (host, port) = match host_port.split_once(':') {
        Some((h, p)) => (h, p.parse::<u16>().unwrap_or(80)),
        None => (host_port, 80),
    };
    let mut stream = TcpStream::connect((host, port)).ok()?;
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes()).ok()?;
    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).ok()?;
    let body_start = resp.windows(4).position(|w| w == b"\r\n\r\n")? + 4;
    Some(resp[body_start..].to_vec())
}

pub fn fetch_live_entropy(sources: &[String]) -> Vec<u8> {
    let mut combined_raw = Vec::new();

    for url in sources {
        if let Some(body) = http_get_body(url) {
            combined_raw.extend_from_slice(&body);
            continue;
        }
        #[cfg(feature = "ledger")]
        if let Ok(data) = its_ledger::fetch_blockchain_latest_hash() {
            combined_raw.extend_from_slice(data.as_bytes());
        }
    }

    if combined_raw.is_empty() && !sources.is_empty() {
        println!("Warning: cover E-sources unreachable; using local fallback entropy.");
        combined_raw.extend_from_slice(b"DEFAULT_FALLBACK_PUBLIC_TELEMETRY_DATA_FOR_ITS_SHADOW_NET");
    }

    combined_raw
}

pub fn verify_otm_tag(
    m: FieldElement,
    k_mac: FieldElement,
    nonce: FieldElement,
    tag: FieldElement,
) -> bool {
    #[cfg(feature = "otm")]
    {
        return bool::from(verify_public_otm_tag(k_mac, m, nonce, tag));
    }
    #[cfg(not(feature = "otm"))]
    {
        let _ = (m, k_mac, nonce, tag);
        eprintln!("Warning: OTM verification disabled (build without `otm` feature).");
        false
    }
}
