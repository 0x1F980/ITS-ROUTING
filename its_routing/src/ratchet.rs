use std::path::Path;

use crate::config::CryptoConfig;

pub(crate) fn demo_aeh_seed(crypto: &CryptoConfig) -> [u8; 32] {
    let mut s = [0u8; 32];
    s[0..4].copy_from_slice(&crypto.stealth_anchor.to_be_bytes());
    s[4..8].copy_from_slice(&crypto.stealth_whitening_factor.to_be_bytes());
    s
}

/// Pool/AEH production path: require a valid 32-byte ratchet seed file.
pub(crate) fn resolve_pool_ratchet_seed(seed_file: &Path) -> Option<[u8; 32]> {
    if seed_file.as_os_str().is_empty() {
        println!(
            "Error: pool transport requires --ratchet-seed-file (exactly 32 bytes). \
             Use ITS-KeyManagement or export-ratchet-seed."
        );
        return None;
    }
    match std::fs::read(seed_file) {
        Ok(data) if data.len() == 32 => {
            let mut seed = [0u8; 32];
            seed.copy_from_slice(&data);
            Some(seed)
        }
        Ok(data) => {
            println!(
                "Error: ratchet seed file must be exactly 32 bytes (got {}).",
                data.len()
            );
            None
        }
        Err(e) => {
            println!("Error: could not read ratchet seed file: {e:?}");
            None
        }
    }
}

/// Dev/demo fallback — not used on pool path.
pub(crate) fn resolve_aeh_ratchet_seed(crypto: &CryptoConfig, seed_file: &Path) -> [u8; 32] {
    if let Some(seed) = resolve_pool_ratchet_seed(seed_file) {
        return seed;
    }
    if !seed_file.as_os_str().is_empty() {
        println!("Warning: invalid ratchet seed; using non-production demo fallback.");
    } else {
        println!(
            "Note: no --ratchet-seed-file; using anchor+whitening demo seed (non-production)."
        );
    }
    demo_aeh_seed(crypto)
}
