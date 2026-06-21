//! Time-lock puzzle generation, unlock, and deniability.

use std::collections::HashMap;
use std::path::PathBuf;

use its_self_enclosed_timelock::field_arith::FieldElement as TlFieldElement;
use its_self_enclosed_timelock::field_arith::MODULUS as TL_MODULUS;
use its_self_enclosed_timelock::{GenerateError, SssTimeLock};

use crate::stdio;

/// Bridges `/dev/urandom` into the standalone time-lock crate's RNG trait.
struct TimelockRng;

impl its_self_enclosed_timelock::SecureRandom for TimelockRng {
    type Error = std::io::Error;

    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Self::Error> {
        use std::io::Read;
        std::fs::File::open("/dev/urandom")?.read_exact(dest)
    }
}

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
            transitions_1: puzzle
                .transitions_1
                .iter()
                .map(|v| v.iter().map(|f| f.value() as u32).collect())
                .collect(),
            transitions_2: puzzle
                .transitions_2
                .iter()
                .map(|v| v.iter().map(|f| f.value() as u32).collect())
                .collect(),
            encrypted_payload: puzzle.encrypted_payload.iter().map(|f| f.value() as u32).collect(),
        }
    }

    fn to_core(&self) -> SssTimeLock {
        SssTimeLock {
            x: self.x,
            m: self.m,
            t: self.t,
            initial_share_1: self
                .initial_share_1
                .iter()
                .map(|&v| TlFieldElement::new(v))
                .collect(),
            transitions_1: self
                .transitions_1
                .iter()
                .map(|v| v.iter().map(|&v| TlFieldElement::new(v)).collect())
                .collect(),
            transitions_2: self
                .transitions_2
                .iter()
                .map(|v| v.iter().map(|&v| TlFieldElement::new(v)).collect())
                .collect(),
            encrypted_payload: self
                .encrypted_payload
                .iter()
                .map(|&v| TlFieldElement::new(v))
                .collect(),
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
                        initial_share_1.push(
                            part.trim()
                                .parse::<u32>()
                                .map_err(|_| "Failed to parse initial share element")?,
                        );
                    }
                }
            } else if key == "encrypted_payload" {
                if !val.is_empty() {
                    for part in val.split(',') {
                        encrypted_payload.push(
                            part.trim()
                                .parse::<u32>()
                                .map_err(|_| "Failed to parse payload element")?,
                        );
                    }
                }
            } else if key.starts_with("transitions_1_block_") {
                let idx_str = key.trim_start_matches("transitions_1_block_");
                let block_idx = idx_str
                    .parse::<usize>()
                    .map_err(|_| "Failed to parse trans1 block index")?;
                let mut vals = Vec::new();
                if !val.is_empty() {
                    for part in val.split(',') {
                        vals.push(
                            part.trim()
                                .parse::<u32>()
                                .map_err(|_| "Failed to parse trans1 element")?,
                        );
                    }
                }
                trans1_map.insert(block_idx, vals);
            } else if key.starts_with("transitions_2_block_") {
                let idx_str = key.trim_start_matches("transitions_2_block_");
                let block_idx = idx_str
                    .parse::<usize>()
                    .map_err(|_| "Failed to parse trans2 block index")?;
                let mut vals = Vec::new();
                if !val.is_empty() {
                    for part in val.split(',') {
                        vals.push(
                            part.trim()
                                .parse::<u32>()
                                .map_err(|_| "Failed to parse trans2 element")?,
                        );
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

pub fn run_time_lock(file_path: PathBuf, epochs: usize, out_path: PathBuf) {
    let mut rng = TimelockRng;
    let quiet = stdio::is_stdio(&out_path);
    stdio::log_status(
        quiet,
        &format!("Loading document for time-locking: {:?}", file_path),
    );

    let message_bytes = match stdio::read_bytes(&file_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: Could not read input: {:?}", e);
            return;
        }
    };

    stdio::log_status(
        quiet,
        &format!("Generating hybrid SSS-Chained Time-Lock over {} epochs...", epochs),
    );

    match SssTimeLock::generate(&message_bytes, epochs, &mut rng) {
        Ok(puzzle) => {
            let text_puzzle = TimeLockText::from_core(&puzzle);
            let serialized = text_puzzle.serialize();

            if let Err(e) = stdio::write_bytes(&out_path, serialized.as_bytes()) {
                eprintln!("Error: Could not write time-lock: {:?}", e);
                return;
            }

            stdio::log_status(quiet, "Time-lock generated successfully!");
            if !quiet {
                println!("- Modulus M: {}", puzzle.m);
                println!("- Base x: {}", puzzle.x);
                println!("- Saved to: {:?}", out_path);
            }
        }
        Err(GenerateError::InvalidInput) => {
            eprintln!("Error: Invalid parameters (empty file or epochs=0).");
        }
        Err(GenerateError::Rng(e)) => {
            eprintln!("Error during entropy collection: {:?}", e);
        }
    }
}

pub fn run_time_unlock(puzzle_path: PathBuf, out_path: PathBuf) {
    let quiet = stdio::is_stdio(&out_path);
    stdio::log_status(
        quiet,
        &format!("Loading time-locked puzzle from: {:?}", puzzle_path),
    );

    let puzzle_content = match stdio::read_text(&puzzle_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error: Could not read time-lock: {:?}", e);
            return;
        }
    };

    let text_puzzle = match TimeLockText::deserialize(&puzzle_content) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error: Invalid time-lock file format: {:?}", e);
            return;
        }
    };

    let puzzle = text_puzzle.to_core();

    stdio::log_status(
        quiet,
        &format!("Starting time detour ({} epochs)...", puzzle.t),
    );

    let start_time = std::time::Instant::now();

    match puzzle.solve() {
        Ok(decrypted_bytes) => {
            let duration = start_time.elapsed();
            stdio::log_status(quiet, &format!("Time-lock unlocked in: {:.2?}", duration));

            if let Err(e) = stdio::write_bytes(&out_path, &decrypted_bytes) {
                eprintln!("Error: Could not write decrypted output: {:?}", e);
                return;
            }

            if !quiet {
                println!("Message decrypted and saved to: {:?}", out_path);
            }
        }
        Err(_) => {
            eprintln!("Error: Could not decrypt time-lock (possibly corrupt data).");
        }
    }
}

pub fn run_time_deny(puzzle_path: PathBuf, decoy_msg: String, out_path: PathBuf) {
    let quiet = stdio::is_stdio(&out_path);
    stdio::log_status(
        quiet,
        &format!(
            "Loading time-locked puzzle for deniability test: {:?}",
            puzzle_path
        ),
    );

    let puzzle_content = match stdio::read_text(&puzzle_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error: Could not read time-lock: {:?}", e);
            return;
        }
    };

    let text_puzzle = match TimeLockText::deserialize(&puzzle_content) {
        Ok(p) => p,
        Err(e) => {
            println!("Error: Invalid time-lock file format: {:?}", e);
            return;
        }
    };

    let puzzle = text_puzzle.to_core();
    let decoy_bytes = decoy_msg.as_bytes();

    if decoy_bytes.len() != puzzle.initial_share_1.len() {
        println!(
            "Warning: Cover story must be exactly the same length as the encrypted payload ({} bytes).",
            puzzle.initial_share_1.len()
        );
        println!("Cover story will be truncated or padded to match the length.");
    }

    let mut padded_decoy = decoy_bytes.to_vec();
    padded_decoy.resize(puzzle.initial_share_1.len(), b' ');

    println!("Performing deniability transposition over SSS transition matrix...");
    println!("This proves the cover story is mathematically 100% consistent with the transition vectors!");

    let mut cur = puzzle.x as u128;
    for _ in 0..puzzle.t {
        cur = (cur * cur) % (puzzle.m as u128);
    }
    let y = cur as u64;

    let mut current_share_2 = Vec::with_capacity(puzzle.initial_share_1.len());
    for idx in 0..puzzle.initial_share_1.len() {
        let s2_0_raw = ((y as u128 + idx as u128) % (TL_MODULUS as u128)) as u32;
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

    let alt_text_puzzle = TimeLockText::from_core(&alt_puzzle);
    let serialized_alt = alt_text_puzzle.serialize();

    if let Err(e) = stdio::write_bytes(&out_path, serialized_alt.as_bytes()) {
        eprintln!("Error: Could not save alternative time-lock: {:?}", e);
        return;
    }

    stdio::log_status(quiet, "Success! Alternative time-lock generated.");
    if !quiet {
        println!("Saved to: {:?}", out_path);
        println!(
            "Cover story on unlock: \"{}\"",
            String::from_utf8_lossy(&padded_decoy).trim()
        );
    }
}
