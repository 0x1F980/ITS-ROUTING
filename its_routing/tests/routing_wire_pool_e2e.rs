//! Full routing + wire E2E: encrypt → UES pool send → receive → decrypt.
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

use its_routing::config::parse_config;
use its_routing::client::{run_client_receive, run_client_send};
use its_routing::ridges::fingerprint_erasure::FingerprintErasureSendOptions;

fn its_asymmetric_bin() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("ITS_ASYMMETRIC_BIN") {
        let path = PathBuf::from(p);
        if path.is_file() {
            return Some(path);
        }
    }
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let c = dir.join("its_asymmetric");
        if c.is_file() {
            return Some(c);
        }
    }
    None
}

#[test]
fn routing_wire_pool_e2e() {
    let its = match its_asymmetric_bin() {
        Some(p) => p,
        None => {
            eprintln!("skip routing_wire_pool_e2e: its_asymmetric not on PATH");
            return;
        }
    };

    let dir = tempfile::tempdir().unwrap();
    let keys = dir.path().join("bob");
    let pool_dir = dir.path().join("pool");
    std::fs::create_dir_all(&keys).unwrap();
    std::fs::create_dir_all(&pool_dir).unwrap();
    let ratchet = dir.path().join("ratchet.seed");
    std::fs::write(&ratchet, &[0x42u8; 32]).unwrap();
    assert!(Command::new(&its)
        .arg("keygen")
        .arg("--out-dir")
        .arg(&keys)
        .status()
        .unwrap()
        .success());

    let msg = b"routing-wire-e2e-payload";
    let plain = dir.path().join("plain.txt");
    let wire = dir.path().join("msg.wire");
    let received = dir.path().join("recv.wire");
    let out = dir.path().join("out.txt");
    std::fs::write(&plain, msg).unwrap();

    assert!(Command::new(&its)
        .arg("encrypt")
        .arg("--pk")
        .arg(keys.join("public.key"))
        .arg("--in")
        .arg(&plain)
        .arg("--out")
        .arg(&wire)
        .status()
        .unwrap()
        .success());

    let cfg = format!(
        r#"
[node]
id = 4
port = 28404
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
tick_rate_ms = 50
payload_size_elements = 16
[routing_table]
4 = "127.0.0.1:28404"
[aeh]
entropy_sources = []
[fingerprint_erasure]
require_on_file_send = false
require_otp = false
require_chaff = false
[pool]
transport_mode = "pool"
pool_file = "{pool}"
epoch_interval_ms = 50
sss_k = 2
sss_n = 3
"#,
        pool = pool_dir.display()
    );
    let config = parse_config(&cfg).expect("config");

    let recv_cfg = config.clone();
    let recv_wire = received.clone();
    let recv_ratchet = ratchet.clone();
    let receiver = thread::spawn(move || {
        run_client_receive(
            recv_cfg,
            false,
            true,
            true,
            recv_ratchet,
            Some(recv_wire),
            15,
            None,
        );
    });

    thread::sleep(Duration::from_millis(100));

    run_client_send(
        config,
        String::new(),
        wire.clone(),
        4,
        false,
        false,
        true,
        ratchet,
        FingerprintErasureSendOptions::default(),
    );

    thread::sleep(Duration::from_millis(500));

    receiver.join().unwrap();
    assert!(received.is_file(), "receiver did not write output");

    assert!(Command::new(&its)
        .arg("decrypt")
        .arg("--pk")
        .arg(keys.join("public.key"))
        .arg("--sk")
        .arg(keys.join("secret.key"))
        .arg("--in")
        .arg(&received)
        .arg("--out")
        .arg(&out)
        .status()
        .unwrap()
        .success());

    assert_eq!(std::fs::read(&out).unwrap(), msg);
}
