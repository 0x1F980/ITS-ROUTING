use std::path::PathBuf;

use crate::client::{run_client_receive, run_client_send};
use crate::config::parse_config;
use crate::pool_mailbox::PoolMailbox;
#[cfg(feature = "dev-onion-mix")]
use crate::daemon::run_node;
use crate::ridges;

pub fn run() {
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
            println!("Could not read config file: {}. Using default configuration.", config_path);
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

            [pool]
            transport_mode = "pool"
            epoch_interval_ms = 100
            cell_size_L = 4096
            pool_file = ".its-pool"
            sss_k = 2
            sss_n = 3
            fountain_enabled = false
            "#.to_string()
        }
    };

    let mut config = parse_config(&config_content).expect("Invalid configuration format");

    let subcommand = command_args[0].as_str();
    match subcommand {
        "start-node" => {
            #[cfg(not(feature = "dev-onion-mix"))]
            {
                println!("Error: start-node requires dev-onion-mix feature (not in production pool build).");
                return;
            }
            #[cfg(feature = "dev-onion-mix")]
            {
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
        }
        "client-send" => {
            let mut msg = String::new();
            let mut file = PathBuf::new();
            let mut dest = 1;
            let mut aeh = false;
            let mut continuous = false;
            let mut pool = false;
            let mut ratchet_seed_file = PathBuf::new();
            let mut fe = ridges::fingerprint_erasure::FingerprintErasureSendOptions::default();

            let mut s_idx = 1;
            while s_idx < command_args.len() {
                #[cfg(not(feature = "fingerprint-erasure"))]
                if ridges::fingerprint_erasure::is_fe_flag(&command_args[s_idx]) {
                    ridges::fingerprint_erasure::reject_fe_flag(&command_args[s_idx]);
                }

                if (command_args[s_idx] == "-m" || command_args[s_idx] == "--msg") && s_idx + 1 < command_args.len() {
                    msg = command_args[s_idx + 1].clone();
                    s_idx += 2;
                } else if (command_args[s_idx] == "-f" || command_args[s_idx] == "--file") && s_idx + 1 < command_args.len() {
                    file = PathBuf::from(&command_args[s_idx + 1]);
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
                } else if command_args[s_idx] == "--pool" {
                    pool = true;
                    s_idx += 1;
                } else if command_args[s_idx] == "--no-pool" {
                    config.pool.transport_mode = "dev".to_string();
                    s_idx += 1;
                } else if command_args[s_idx] == "--ratchet-seed-file" && s_idx + 1 < command_args.len() {
                    ratchet_seed_file = PathBuf::from(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else {
                    #[cfg(feature = "fingerprint-erasure")]
                    {
                        if !ridges::fingerprint_erasure::try_parse_client_send_arg(
                            &command_args,
                            &mut s_idx,
                            &mut fe,
                        ) {
                            s_idx += 1;
                        }
                    }
                    #[cfg(not(feature = "fingerprint-erasure"))]
                    {
                        s_idx += 1;
                    }
                }
            }
            #[cfg(feature = "fingerprint-erasure")]
            {
                let fe_cfg = ridges::fingerprint_erasure::ClientSendFeConfig {
                    default_pad: &config.fingerprint_erasure.default_pad,
                    require_otp: config.fingerprint_erasure.require_otp,
                    require_chaff: config.fingerprint_erasure.require_chaff,
                    chaff_enabled: config.traffic.constant_rate_chaff_enabled,
                    require_on_file_send: config.fingerprint_erasure.require_on_file_send,
                    file_path: file.to_str().unwrap_or(""),
                };
                if !ridges::fingerprint_erasure::finalize_client_send_fe(&mut fe, &fe_cfg) {
                    return;
                }
            }
            run_client_send(config, msg, file, dest, aeh, continuous, pool, ratchet_seed_file, fe);
        }
        "client-receive" => {
            let mut aeh = false;
            let mut continuous = false;
            let mut pool = false;
            let mut ratchet_seed_file = PathBuf::new();
            let mut out_path: Option<PathBuf> = None;
            let mut timeout_secs = 30u64;
            let mut mailbox: Option<PoolMailbox> = None;

            let mut s_idx = 1;
            while s_idx < command_args.len() {
                if command_args[s_idx] == "--aeh" {
                    aeh = true;
                    s_idx += 1;
                } else if command_args[s_idx] == "--continuous" {
                    continuous = true;
                    s_idx += 1;
                } else if command_args[s_idx] == "--pool" {
                    pool = true;
                    s_idx += 1;
                } else if command_args[s_idx] == "--no-pool" {
                    config.pool.transport_mode = "dev".to_string();
                    s_idx += 1;
                } else if command_args[s_idx] == "--mailbox-strict" {
                    let mut mb = mailbox.take().unwrap_or_default();
                    mb.strict = true;
                    mailbox = Some(mb);
                    s_idx += 1;
                } else if (command_args[s_idx] == "--mailbox-fingerprint"
                    || command_args[s_idx] == "--mailbox-fp")
                    && s_idx + 1 < command_args.len()
                {
                    let fp = &command_args[s_idx + 1];
                    mailbox = PoolMailbox::from_fingerprint_hex(fp).or_else(|| {
                        std::fs::read(fp).ok().map(|b| PoolMailbox::from_fingerprint_bytes(&b))
                    });
                    s_idx += 2;
                } else if command_args[s_idx] == "--ratchet-seed-file" && s_idx + 1 < command_args.len() {
                    ratchet_seed_file = PathBuf::from(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else if (command_args[s_idx] == "-o" || command_args[s_idx] == "--out") && s_idx + 1 < command_args.len() {
                    out_path = Some(PathBuf::from(&command_args[s_idx + 1]));
                    s_idx += 2;
                } else if command_args[s_idx] == "--timeout-secs" && s_idx + 1 < command_args.len() {
                    timeout_secs = command_args[s_idx + 1].parse().unwrap_or(30);
                    s_idx += 2;
                } else {
                    s_idx += 1;
                }
            }
            run_client_receive(
                config,
                aeh,
                continuous,
                pool,
                ratchet_seed_file,
                out_path,
                timeout_secs,
                mailbox,
            );
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
            #[cfg(feature = "timelock")]
            ridges::timelock::run_time_lock(file, epochs, out);
            #[cfg(not(feature = "timelock"))]
            ridges::missing_ridge("timelock");
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
            #[cfg(feature = "timelock")]
            ridges::timelock::run_time_unlock(puzzle, out);
            #[cfg(not(feature = "timelock"))]
            ridges::missing_ridge("timelock");
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
            #[cfg(feature = "timelock")]
            ridges::timelock::run_time_deny(puzzle, decoy, out);
            #[cfg(not(feature = "timelock"))]
            ridges::missing_ridge("timelock");
        }
        "fingerprint-erasure" => {
            #[cfg(feature = "fingerprint-erasure")]
            {
            let mut file = PathBuf::new();
            let mut out = PathBuf::new();
            let mut out_otp = PathBuf::new();
            let mut pad = PathBuf::new();
            let mut delta = None;
            let mut output_format = its_fingerprint_erasure::OutputFormat::Auto;

            let mut s_idx = 1;
            while s_idx < command_args.len() {
                if (command_args[s_idx] == "-f" || command_args[s_idx] == "--file" || command_args[s_idx] == "--in")
                    && s_idx + 1 < command_args.len()
                {
                    file = PathBuf::from(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else if (command_args[s_idx] == "-o" || command_args[s_idx] == "--out") && s_idx + 1 < command_args.len() {
                    out = PathBuf::from(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else if command_args[s_idx] == "--out-otp" && s_idx + 1 < command_args.len() {
                    out_otp = PathBuf::from(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else if command_args[s_idx] == "--pad" && s_idx + 1 < command_args.len() {
                    pad = PathBuf::from(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else if command_args[s_idx] == "--delta" && s_idx + 1 < command_args.len() {
                    delta = command_args[s_idx + 1].parse().ok();
                    s_idx += 2;
                } else if command_args[s_idx] == "--format" && s_idx + 1 < command_args.len() {
                    output_format = ridges::fingerprint_erasure::parse_fe_output_format(&command_args[s_idx + 1]);
                    s_idx += 2;
                } else {
                    s_idx += 1;
                }
            }
            ridges::fingerprint_erasure::run_fingerprint_erasure(file, out, out_otp, pad, delta, output_format);
            }
            #[cfg(not(feature = "fingerprint-erasure"))]
            ridges::missing_ridge("fingerprint-erasure");
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
                println!("Error: Message must not be empty. Use -m, --msg <text>");
                return;
            }
            let k = threshold_k.unwrap_or(config.crypto.threshold_k);
            let n = total_shares_n.unwrap_or(config.crypto.total_shares_n);
            #[cfg(feature = "hardware")]
            ridges::analog::run_export_share(msg, k, n);
            #[cfg(not(feature = "hardware"))]
            ridges::missing_ridge("hardware");
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
                        println!("Error: Could not read file {:?}: {:?}", fp, e);
                        return;
                    }
                }
            }

            if shares_input.is_empty() {
                println!("Error: No analog shares found. Pass them as arguments, or load with -f, --file <path>");
                return;
            }

            let k = threshold_k.unwrap_or(config.crypto.threshold_k);
            #[cfg(feature = "hardware")]
            ridges::analog::run_import_share(shares_input, k);
            #[cfg(not(feature = "hardware"))]
            ridges::missing_ridge("hardware");
        }
        _ => {
            print_usage();
        }
    }
}

fn print_usage() {
    println!("ITS-routing — UES Monocell Pool transport (v2.0 prod default)");
    println!("PATH \"-\" = stdin/stdout on time-lock and fingerprint-erasure (see ITS-routing_PIPE.md).");
    println!("Production: pool → cover harvest → optional its-pool-proxy. See QUICKSTART.md.");
    println!("Usage:");
    println!("  its-routing [subcommand] [options]");
    println!("\nSubcommands:");
    println!("  client-send     Publish Shannon ITS wire to UES Monocell Pool (default when transport_mode=pool)");
    println!("                  -m, --msg <text>        Message string to send");
    println!("                  -f, --file <path>       File payload to send (optional vs --msg)");
    println!("                  -d, --dest <id>         Destination Node Field Element ID (dev onion only)");
    println!("                  --fingerprint-erasure   Optional Γ v3/v4 universal CR-NF before send (max security default when enabled)");
    println!("                  --fe-strict, --strict           Opt-in strict policy: explicit kind, deny Raw");
    println!("                  --fe-strict-stack, --strict-stack  Extended preset (v0.8 default with --fingerprint-erasure)");
    println!("                  --fe-permissive, --permissive     v5 permissive Γ escape (OTP optional)");
    println!("                  --fe-domain, --domain   discrete|continuous (must match --fe-kind)");
    println!("                  --fe-kind, --kind       text|image|audio|pdf|code (required in strict unless -f ext known)");
    println!("                  --fe-delta, --delta <N> Pixel quantization threshold");
    println!("                  --fe-mode, --mode       standard|extended|minimal (aliases: max, annihilator, balanced)");
    println!("                  --fe-lexicon, --lexicon da-en|off (default da-en in max mode)");
    println!("                  --fe-dct-q, --dct-q      DCT AC quantization (max mode)");
    println!("                  --fe-sigma-delta        SVD sigma quantization (max mode)");
    println!("                  --fe-lab-delta-ab       LAB chroma quantization (max mode)");
    println!("                  --fe-no-midband-zero    Disable midband AC zeroing");
    println!("                  --fe-format, --format   auto|sem1|png|txt|bin|wav|code");
    println!("                  --fe-pad, --pad <path>  OTP wire payload on send (requires --fingerprint-erasure)");
    println!("                  --pool                  UES Monocell Pool transport (default when transport_mode=pool)");
    println!("                  --no-pool               Disable pool default; use dev transport features");
    println!("                  --aeh                   Manual AEH last-resort (no auto-switch from pool)");
    println!("                  --continuous            Enable continuous background decoy chaffing schedule loop");
    println!("                  --ratchet-seed-file <path>  32-byte TransportOtpRatchet OTP seed (from ITS-KeyManagement export)");
    println!("  client-receive  Harvest pool + cover entropy; reconstruct wire (default when transport_mode=pool)");
    println!("                  --pool                  UES Monocell Pool harvest (default when transport_mode=pool)");
    println!("                  --no-pool               Disable pool default");
    println!("                  --aeh                   Manual AEH scan (last-resort only)");
    println!("                  --continuous            Epoch-loop receive until wire found");
    println!("                  --ratchet-seed-file <path>  32-byte TransportOtpRatchet OTP seed (from ITS-KeyManagement export)");
    println!("                  --mailbox-fingerprint <hex|file>  PoolMailbox contact hint (ciphertext scope)");
    println!("                  --mailbox-strict                    Reject reconstructions failing wire/OTM gate");
    println!("  start-node      [dev-onion-mix only] Active onion routing daemon — not prod default");
    println!("                  -p, --port <port>       Port to bind the listener to");
    println!("                  -r, --chaff-rate <ms>   Continuous dummy chaff loop timing");
    println!("  time-lock       Generates a local hybrid deniable time-lock puzzle over a file");
    println!("                  -f, --file <path>       Target document to lock");
    println!("                  -e, --epochs <count>    Number of sequential squaring delay rounds (default 1000)");
    println!("                  -o, --out <path>        Output path to write locked puzzle (.its)");
    println!("  time-unlock     Solves modular squarings sequentially on CPU to decrypt a puzzle");
    println!("                  -p, --puzzle <path>     Target puzzle .its file");
    println!("                  -o, --out <path>        Output decrypted file path");
    println!("  time-deny       Asserts a decoy cover-story message to solve the puzzle to alternative 'truth'");
    println!("                  -p, --puzzle <path>     Target puzzle .its file");
    println!("                  -d, --decoy <text>      Harmless decoy message of equal length");
    println!("                  -o, --out <path>        Output alternative decrypted file path");
    println!("  fingerprint-erasure  Standalone offline Γ (+ optional OTP to files)");
    println!("                  -f, --file, --in <path> Input file");
    println!("                  -o, --out <path>        Normalized file for Bob (after otp-unmask)");
    println!("                  --pad <path>            Offline pad (required with --out-otp)");
    println!("                  --out-otp <path>        ITS-WIR1 wire ciphertext for Eve-facing transport");
    println!("                  --delta <N>             Quantization threshold");
    println!("                  --format <fmt>          auto|sem1|png|txt|bin");
    println!("  client-export-share Serialize Shamir shares into physical character strings (analog export)");
    println!("                  -m, --msg <text>        Target secret message to split");
    println!("                  -k, --threshold <k>     Override threshold k");
    println!("                  -n, --shares <n>        Override total shares n");
    println!("  client-export-share Serialize Shamir shares into physical character strings (analog export)");
    println!("                  -m, --msg <text>        Target secret message to split");
    println!("                  -k, --threshold <k>     Override threshold k");
    println!("                  -n, --shares <n>        Override total shares n");
    println!("  client-import-share Reconstruct Shamir secret from physical character strings (analog import)");
    println!("                  -f, --file <path>       File containing physical share strings (one per line)");
    println!("                  -k, --threshold <k>     Override threshold k");
    println!("                  [shares]                Provide physical share strings directly as arguments");
    println!("  Operator identity (vault, contacts, duress): use ITS-KeyManagement (its-km on PATH).");
}
