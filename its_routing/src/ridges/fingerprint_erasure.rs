//! Fingerprint erasure (Γ) send options and CLI ridge.

use std::path::PathBuf;

use crate::stdio;

#[derive(Clone)]
pub struct FingerprintErasureSendOptions {
    pub enabled: bool,
    pub delta: Option<u32>,
    pub output_format: its_fingerprint_erasure::OutputFormat,
    pub pad_path: PathBuf,
    pub otp_wire: bool,
    pub mode: its_fingerprint_erasure::ErasureMode,
    pub lexicon: its_fingerprint_erasure::LexiconMode,
    pub dct_q_ac: Option<u32>,
    pub dct_zero_midband: Option<bool>,
    pub sigma_delta: Option<u32>,
    pub lab_delta_ab: Option<u32>,
    pub strict: bool,
    pub strict_stack: bool,
    pub permissive: bool,
    pub explicit_mode: bool,
    pub declared_kind: Option<its_fingerprint_erasure::SemanticKind>,
    pub declared_domain: Option<its_fingerprint_erasure::SemanticDomain>,
}

impl Default for FingerprintErasureSendOptions {
    fn default() -> Self {
        Self {
            enabled: false,
            delta: None,
            output_format: its_fingerprint_erasure::OutputFormat::Auto,
            pad_path: PathBuf::new(),
            otp_wire: false,
            mode: its_fingerprint_erasure::ErasureMode::Standard,
            lexicon: its_fingerprint_erasure::LexiconMode::DaEn,
            dct_q_ac: None,
            dct_zero_midband: None,
            sigma_delta: None,
            lab_delta_ab: None,
            strict: false,
            strict_stack: false,
            permissive: false,
            explicit_mode: false,
            declared_kind: None,
            declared_domain: None,
        }
    }
}

impl FingerprintErasureSendOptions {
    pub fn to_erasure_options(&self) -> its_fingerprint_erasure::ErasureOptions {
        if self.strict_stack {
            let mode = if self.explicit_mode {
                self.mode
            } else {
                its_fingerprint_erasure::ErasureMode::Extended
            };
            return its_fingerprint_erasure::ErasureOptions {
                delta: self.delta,
                output_format: self.output_format,
                mode,
                lexicon: self.lexicon,
                dct_q_ac: self.dct_q_ac,
                dct_zero_midband: self.dct_zero_midband,
                sigma_delta: self.sigma_delta,
                lab_delta_ab: self.lab_delta_ab,
                declared_kind: self.declared_kind,
                declared_domain: self.declared_domain,
                ..its_fingerprint_erasure::ErasureOptions::strict_stack()
            };
        }
        let (policy, allow_raw) = if self.strict {
            (
                its_fingerprint_erasure::ErasurePolicy::Strict,
                false,
            )
        } else {
            (
                its_fingerprint_erasure::ErasurePolicy::Permissive,
                true,
            )
        };
        its_fingerprint_erasure::ErasureOptions {
            delta: self.delta,
            output_format: self.output_format,
            mode: self.mode,
            lexicon: self.lexicon,
            dct_q_ac: self.dct_q_ac,
            dct_zero_midband: self.dct_zero_midband,
            sigma_delta: self.sigma_delta,
            lab_delta_ab: self.lab_delta_ab,
            policy,
            declared_kind: self.declared_kind,
            declared_domain: self.declared_domain,
            allow_raw,
        }
    }
}

pub fn prepare_send_payload(
    raw: &[u8],
    fe: &FingerprintErasureSendOptions,
) -> Result<Vec<u8>, its_fingerprint_erasure::FeError> {
    if !fe.enabled {
        return Ok(raw.to_vec());
    }

    if fe.otp_wire && fe.pad_path.as_os_str().is_empty() {
        return Err(its_fingerprint_erasure::FeError::InvalidPad(
            "pad path required for OTP wire send".into(),
        ));
    }

    let gamma_out = its_fingerprint_erasure::erase_provenance(raw, fe.to_erasure_options())?;

    if fe.otp_wire {
        let mut pad = its_fingerprint_erasure::PadFile::open(&fe.pad_path)?;
        its_fingerprint_erasure::otp_mask(&gamma_out.bytes, &mut pad)
    } else {
        Ok(gamma_out.bytes)
    }
}

pub fn parse_fe_mode(s: &str) -> its_fingerprint_erasure::ErasureMode {
    match s.to_ascii_lowercase().as_str() {
        "minimal" | "balanced" => its_fingerprint_erasure::ErasureMode::Minimal,
        "extended" | "annihilator" | "annihilate" => its_fingerprint_erasure::ErasureMode::Extended,
        "standard" | "max" | "max-security" => its_fingerprint_erasure::ErasureMode::Standard,
        _ => its_fingerprint_erasure::ErasureMode::Standard,
    }
}

pub fn parse_fe_lexicon(s: &str) -> its_fingerprint_erasure::LexiconMode {
    match s.to_ascii_lowercase().as_str() {
        "off" => its_fingerprint_erasure::LexiconMode::Off,
        _ => its_fingerprint_erasure::LexiconMode::DaEn,
    }
}

pub fn parse_fe_output_format(s: &str) -> its_fingerprint_erasure::OutputFormat {
    match s.to_ascii_lowercase().as_str() {
        "sem1" => its_fingerprint_erasure::OutputFormat::Sem1,
        "png" => its_fingerprint_erasure::OutputFormat::Png,
        "txt" => its_fingerprint_erasure::OutputFormat::Txt,
        "bin" => its_fingerprint_erasure::OutputFormat::Bin,
        "wav" => its_fingerprint_erasure::OutputFormat::Wav,
        "code" => its_fingerprint_erasure::OutputFormat::Code,
        _ => its_fingerprint_erasure::OutputFormat::Auto,
    }
}

/// Parse one client-send fingerprint-erasure flag. Returns `true` if `arg` was consumed.
pub fn try_parse_client_send_arg(
    args: &[String],
    s_idx: &mut usize,
    fe: &mut FingerprintErasureSendOptions,
) -> bool {
    let arg = &args[*s_idx];
    if arg == "--fingerprint-erasure" || arg == "--gamma" {
        fe.enabled = true;
        *s_idx += 1;
        return true;
    }
    if (arg == "--fe-delta" || arg == "--delta") && *s_idx + 1 < args.len() {
        fe.delta = args[*s_idx + 1].parse().ok();
        *s_idx += 2;
        return true;
    }
    if (arg == "--fe-format" || arg == "--format") && *s_idx + 1 < args.len() {
        fe.output_format = parse_fe_output_format(&args[*s_idx + 1]);
        *s_idx += 2;
        return true;
    }
    if (arg == "--fe-pad" || arg == "--pad") && *s_idx + 1 < args.len() {
        fe.pad_path = PathBuf::from(&args[*s_idx + 1]);
        fe.otp_wire = true;
        *s_idx += 2;
        return true;
    }
    if (arg == "--fe-mode" || arg == "--mode") && *s_idx + 1 < args.len() {
        fe.mode = parse_fe_mode(&args[*s_idx + 1]);
        fe.explicit_mode = true;
        *s_idx += 2;
        return true;
    }
    if (arg == "--fe-lexicon" || arg == "--lexicon") && *s_idx + 1 < args.len() {
        fe.lexicon = parse_fe_lexicon(&args[*s_idx + 1]);
        *s_idx += 2;
        return true;
    }
    if (arg == "--fe-dct-q" || arg == "--dct-q") && *s_idx + 1 < args.len() {
        fe.dct_q_ac = args[*s_idx + 1].parse().ok();
        *s_idx += 2;
        return true;
    }
    if arg == "--fe-no-midband-zero" {
        fe.dct_zero_midband = Some(false);
        *s_idx += 1;
        return true;
    }
    if (arg == "--fe-sigma-delta" || arg == "--sigma-delta") && *s_idx + 1 < args.len() {
        fe.sigma_delta = args[*s_idx + 1].parse().ok();
        *s_idx += 2;
        return true;
    }
    if (arg == "--fe-lab-delta-ab" || arg == "--lab-delta-ab") && *s_idx + 1 < args.len() {
        fe.lab_delta_ab = args[*s_idx + 1].parse().ok();
        *s_idx += 2;
        return true;
    }
    if arg == "--fe-strict" || arg == "--strict" {
        fe.strict = true;
        *s_idx += 1;
        return true;
    }
    if arg == "--fe-strict-stack"
        || arg == "--strict-stack"
        || arg == "--fe-uangribelig"
        || arg == "--uangribelig"
    {
        fe.strict_stack = true;
        fe.strict = true;
        *s_idx += 1;
        return true;
    }
    if arg == "--fe-permissive" || arg == "--permissive" {
        #[cfg(feature = "dev-permissive")]
        {
            fe.permissive = true;
            *s_idx += 1;
            return true;
        }
        #[cfg(not(feature = "dev-permissive"))]
        {
            println!("Error: --fe-permissive requires dev-permissive feature.");
            std::process::exit(1);
        }
    }
    if (arg == "--fe-domain" || arg == "--domain") && *s_idx + 1 < args.len() {
        fe.declared_domain =
            its_fingerprint_erasure::parse_semantic_domain(&args[*s_idx + 1]);
        *s_idx += 2;
        return true;
    }
    if (arg == "--fe-kind" || arg == "--kind") && *s_idx + 1 < args.len() {
        fe.declared_kind = its_fingerprint_erasure::parse_semantic_kind(&args[*s_idx + 1]);
        *s_idx += 2;
        return true;
    }
    false
}

pub struct ClientSendFeConfig<'a> {
    pub default_pad: &'a str,
    pub require_otp: bool,
    pub require_chaff: bool,
    pub chaff_enabled: bool,
    pub require_on_file_send: bool,
    pub file_path: &'a str,
}

/// Apply post-parse fingerprint-erasure policy. Returns `false` if client-send should abort.
pub fn finalize_client_send_fe(
    fe: &mut FingerprintErasureSendOptions,
    cfg: &ClientSendFeConfig<'_>,
) -> bool {
    if fe.otp_wire && !fe.pad_path.as_os_str().is_empty() {
        fe.enabled = true;
    }
    if fe.enabled && !fe.permissive {
        fe.strict_stack = true;
        fe.strict = true;
        if !fe.explicit_mode {
            fe.mode = its_fingerprint_erasure::ErasureMode::Extended;
        }
        if fe.pad_path.as_os_str().is_empty() && !cfg.default_pad.is_empty() {
            fe.pad_path = PathBuf::from(cfg.default_pad);
        }
        if cfg.require_otp && fe.pad_path.as_os_str().is_empty() {
            println!(
                "Error: strict stack requires --fe-pad (OTP) or [fingerprint_erasure].default_pad in config."
            );
            return false;
        }
        if !fe.pad_path.as_os_str().is_empty() {
            fe.otp_wire = true;
        }
        if cfg.require_chaff && !cfg.chaff_enabled {
            println!("Error: strict stack requires constant_rate_chaff_enabled=true in config.");
            return false;
        }
        if let Err(e) = its_fingerprint_erasure::validate_send_stack(
            !fe.pad_path.as_os_str().is_empty(),
            cfg.chaff_enabled,
        ) {
            println!("Error: strict stack: {e}");
            return false;
        }
    }
    if (fe.strict || fe.strict_stack) && fe.declared_kind.is_none() && !cfg.file_path.is_empty() {
        fe.declared_kind = its_fingerprint_erasure::kind_from_path(cfg.file_path);
    }
    if (fe.strict || fe.strict_stack) && fe.declared_domain.is_none() && !cfg.file_path.is_empty() {
        fe.declared_domain = its_fingerprint_erasure::domain_from_path(cfg.file_path);
    }
    if cfg.require_on_file_send && !cfg.file_path.is_empty() && !fe.enabled {
        println!("Error: client-send --file requires --fingerprint-erasure (v0.8 strict stack).");
        return false;
    }
    true
}

pub fn run_fingerprint_erasure(
    file_path: PathBuf,
    out_path: PathBuf,
    out_otp_path: PathBuf,
    pad_path: PathBuf,
    delta: Option<u32>,
    output_format: its_fingerprint_erasure::OutputFormat,
) {
    if file_path.as_os_str().is_empty() || out_path.as_os_str().is_empty() {
        eprintln!("Error: fingerprint-erasure requires --file/--in and --out.");
        return;
    }
    let want_otp = !out_otp_path.as_os_str().is_empty();
    if want_otp && pad_path.as_os_str().is_empty() {
        eprintln!("Error: --out-otp requires --pad.");
        return;
    }
    let quiet = stdio::is_stdio(&out_path);

    let input = match stdio::read_bytes(&file_path) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Error: Could not read input: {:?}", e);
            return;
        }
    };

    if pad_path.as_os_str().is_empty() && !want_otp && !quiet {
        println!("Warning: v0.8 recommends OTP — use --pad and --out-otp for strict stack.");
    }

    let path_str = if stdio::is_stdio(&file_path) {
        "stdin"
    } else {
        file_path.to_str().unwrap_or("")
    };
    let declared_kind = its_fingerprint_erasure::kind_from_path(path_str);
    let declared_domain = its_fingerprint_erasure::domain_from_path(path_str);
    let mut opts = its_fingerprint_erasure::ErasureOptions::strict_stack();
    opts.delta = delta;
    opts.output_format = output_format;
    opts.declared_kind = declared_kind;
    opts.declared_domain = declared_domain;

    let gamma_out = match its_fingerprint_erasure::erase_provenance(&input, opts) {
        Ok(outp) => outp,
        Err(e) => {
            eprintln!("Error during provenance erasure: {:?}", e);
            return;
        }
    };

    if let Err(e) = stdio::write_bytes(&out_path, &gamma_out.bytes) {
        eprintln!("Error: Could not write normalized output: {:?}", e);
        return;
    }
    stdio::log_status(quiet, "Provenance erasure completed.");
    if !quiet {
        println!("- Normalized: {:?}", out_path);
    }

    if want_otp {
        let mut pad = match its_fingerprint_erasure::PadFile::open(&pad_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error: Pad file: {:?}", e);
                return;
            }
        };
        match its_fingerprint_erasure::otp_mask(&gamma_out.bytes, &mut pad) {
            Ok(wire) => {
                if let Err(e) = stdio::write_bytes(&out_otp_path, &wire) {
                    eprintln!("Error: Could not write wire: {:?}", e);
                    return;
                }
                if !quiet {
                    println!("- Wire (OTP): {:?}", out_otp_path);
                }
            }
            Err(e) => eprintln!("Error during OTP mask: {:?}", e),
        }
    }
}
