use std::path::Path;

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
