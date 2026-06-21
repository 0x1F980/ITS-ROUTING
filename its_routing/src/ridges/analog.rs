//! Physical/analog SSS share export and import.

use its_hardware::analog_shares::{export_analog_share, import_analog_share};
use its_transport::{fragment_data, reconstruct_data};

use crate::rng::RoutingRng;

pub fn run_export_share(msg: String, k: usize, n: usize) {
    println!("Analog-export: Fragmenting message with k={}, n={}", k, n);

    let mut rng = RoutingRng;
    match fragment_data(msg.as_bytes(), k, n, &mut rng) {
        Ok(shares) => {
            println!("--- REPRODUCIBLE PHYSICAL SSS SHARES (COPYABLE PAPER BLOCKS) ---");
            for share in &shares {
                let encoded = export_analog_share(share);
                println!("{}", encoded);
            }
            println!("----------------------------------------------------------------------");
            println!("Store these strings safely on independent analog media (paper, QR codes, microfilm).");
            println!(
                "Any collection of {} out of these {} strings can fully reconstruct the message.",
                k, n
            );
        }
        Err(_) => {
            println!("Error fragmenting data.");
        }
    }
}

pub fn run_import_share(shares_input: Vec<String>, k: usize) {
    println!(
        "Analog import: Attempting to reconstruct message from {} analog shares (k={}).",
        shares_input.len(),
        k
    );

    let mut parsed_shares = Vec::new();
    for input in &shares_input {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        match import_analog_share(trimmed) {
            Ok(share) => {
                println!("Loaded valid share ID: {}", share.id.value() as u32);
                parsed_shares.push(share);
            }
            Err(e) => {
                println!("Error loading share \"{}\": {}", trimmed, e);
                return;
            }
        }
    }

    if parsed_shares.is_empty() {
        println!("Error: No valid analog shares entered.");
        return;
    }

    match reconstruct_data(&parsed_shares, k) {
        Ok(secret_bytes) => {
            println!("\n--- RECONSTRUCTED SECRET (100% CORRECT) ---");
            println!("{}", String::from_utf8_lossy(&secret_bytes));
            println!("-------------------------------------------------");
        }
        Err(_) => {
            println!(
                "Error: Reconstruction failed. Possibly too few shares (have {}, need k={}), or shares belong to different secrets.",
                parsed_shares.len(),
                k
            );
        }
    }
}

#[cfg(all(test, feature = "hardware"))]
mod cli_analog_tests {
    use super::*;
    use crate::rng::RoutingRng;

    #[test]
    fn test_analog_share_roundtrip() {
        #[cfg(feature = "m61")]
        println!("--- CLI TEST: FEATURE m61 IS ENABLED ---");
        #[cfg(not(feature = "m61"))]
        println!("--- CLI TEST: FEATURE m61 IS DISABLED ---");

        let mut rng = RoutingRng;
        let original_secret = b"Information-Theoretic Secrecy is the ultimate goal!";
        let k = 3;
        let n = 5;
        let shares = fragment_data(original_secret, k, n, &mut rng).unwrap();

        let mut exported_strings = Vec::new();
        for share in &shares {
            let exported = export_analog_share(share);
            exported_strings.push(exported);
        }

        let test_share = &shares[0];
        let test_exported = export_analog_share(test_share);
        println!("EXPORTED HEX STRING: {}", test_exported);
        let test_imported = import_analog_share(&test_exported).unwrap();
        println!(
            "ORIGINAL ID: {}, IMPORTED ID: {}",
            test_share.id.value() as u64,
            test_imported.id.value() as u64
        );
        println!(
            "ORIGINAL LEN: {}, IMPORTED LEN: {}",
            test_share.data_points.len(),
            test_imported.data_points.len()
        );
        for i in 0..test_share.data_points.len() {
            println!(
                "PT [{}]: ORIGINAL={}, IMPORTED={}",
                i,
                test_share.data_points[i].value() as u64,
                test_imported.data_points[i].value() as u64
            );
        }

        assert_eq!(test_share.id.value(), test_imported.id.value());
        assert_eq!(
            test_share.data_points.len(),
            test_imported.data_points.len()
        );
        for i in 0..test_share.data_points.len() {
            assert_eq!(
                test_share.data_points[i].value(),
                test_imported.data_points[i].value()
            );
        }
        let subset_to_import = &exported_strings[0..k];
        let mut imported_shares = Vec::new();
        for s_str in subset_to_import {
            let imported = import_analog_share(s_str).unwrap();
            imported_shares.push(imported);
        }

        let reconstructed = reconstruct_data(&imported_shares, k).unwrap();
        assert_eq!(reconstructed, original_secret);
    }
}
