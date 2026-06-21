//! Optional operational ridges — compile-time features on `its_routing`.

#[cfg(feature = "timelock")]
pub mod timelock;
#[cfg(feature = "fingerprint-erasure")]
pub mod fingerprint_erasure;
#[cfg(not(feature = "fingerprint-erasure"))]
pub mod fingerprint_erasure {
    #[derive(Clone, Default)]
    pub struct FingerprintErasureSendOptions {
        pub enabled: bool,
    }

    pub fn is_fe_flag(arg: &str) -> bool {
        matches!(
            arg,
            "--fingerprint-erasure"
                | "--gamma"
                | "--fe-delta"
                | "--delta"
                | "--fe-format"
                | "--format"
                | "--fe-pad"
                | "--pad"
                | "--fe-mode"
                | "--mode"
                | "--fe-lexicon"
                | "--lexicon"
                | "--fe-dct-q"
                | "--dct-q"
                | "--fe-no-midband-zero"
                | "--fe-sigma-delta"
                | "--sigma-delta"
                | "--fe-lab-delta-ab"
                | "--lab-delta-ab"
                | "--fe-strict"
                | "--strict"
                | "--fe-strict-stack"
                | "--strict-stack"
                | "--fe-uangribelig"
                | "--uangribelig"
                | "--fe-permissive"
                | "--permissive"
                | "--fe-domain"
                | "--domain"
                | "--fe-kind"
                | "--kind"
        )
    }

    pub fn reject_fe_flag(flag: &str) -> ! {
        eprintln!(
            "Error: {flag} requires `fingerprint-erasure` feature.\n\
             Rebuild: cargo build -p its_routing --features fingerprint-erasure"
        );
        std::process::exit(1);
    }

    pub fn prepare_send_payload(
        raw: &[u8],
        _fe: &FingerprintErasureSendOptions,
    ) -> Result<Vec<u8>, &'static str> {
        Ok(raw.to_vec())
    }
}

#[cfg(feature = "hardware")]
pub mod analog;

pub fn missing_ridge(name: &str) -> ! {
    eprintln!(
        "its-routing built without `{name}`.\n\
         Rebuild: cargo build -p its_routing --features {name}\n\
         Or all ridges: cargo build -p its_routing --features full"
    );
    std::process::exit(1);
}
