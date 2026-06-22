//! Valid forward party — mirror whitelist M_valid (Lean: `ValidForwardParty.lean`, `ForwardReceiveGate.lean`).

use std::collections::{HashMap, HashSet};

use crate::availability_ledger::{self, AvailabilityAttackKind};

/// Canonical public pool log: epoch → published cell bytes.
#[derive(Debug, Clone, Default)]
pub struct CanonicalLog {
    cells: HashMap<u64, Vec<u8>>,
}

impl CanonicalLog {
    pub fn get(&self, epoch: u64) -> Option<&[u8]> {
        self.cells.get(&epoch).map(|v| v.as_slice())
    }

    pub fn published_epochs(&self) -> impl Iterator<Item = u64> + '_ {
        let mut keys: Vec<u64> = self.cells.keys().copied().collect();
        keys.sort_unstable();
        keys.into_iter()
    }
}

/// Per-mirror harvest views + canonical log (Lean: `PoolView` + `CanonicalLog`).
#[derive(Debug, Clone, Default)]
pub struct ValidForwardState {
    pub canonical: CanonicalLog,
    mirror_harvests: HashMap<String, HashMap<u64, Vec<u8>>>,
    de_whitelisted: HashSet<String>,
    mirror_actors: HashMap<String, u32>,
}

impl ValidForwardState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register mirror URL → availability-ledger actor id.
    pub fn register_mirror(&mut self, mirror: &str, actor: u32) {
        self.mirror_actors.insert(mirror.to_string(), actor);
    }

    fn mirror_actor(&self, mirror: &str) -> u32 {
        self.mirror_actors
            .get(mirror)
            .copied()
            .unwrap_or_else(|| stable_mirror_actor(mirror))
    }

    pub fn harvest_view(&self, mirror: &str, epoch: u64) -> Option<&[u8]> {
        self.mirror_harvests
            .get(mirror)
            .and_then(|h| h.get(&epoch))
            .map(|v| v.as_slice())
    }

    pub fn is_de_whitelisted(&self, mirror: &str) -> bool {
        self.de_whitelisted.contains(mirror)
    }
}

/// Stable actor id from mirror URL (AvailabilityLedger-compatible).
pub fn stable_mirror_actor(mirror: &str) -> u32 {
    let mut h: u32 = 0x9E37_79B9;
    for b in mirror.bytes() {
        h = h.wrapping_mul(31).wrapping_add(u32::from(b));
    }
    h.max(1)
}

pub fn record_publish(log: &mut CanonicalLog, epoch: u64, cell: &[u8]) {
    log.cells.insert(epoch, cell.to_vec());
}

/// Record one mirror harvest at `epoch`; on mismatch with canonical ⇒ de-whitelist + strike.
pub fn record_harvest(
    state: &mut ValidForwardState,
    mirror: &str,
    epoch: u64,
    cell: Option<&[u8]>,
    window: u64,
) {
    let harvest_map = state
        .mirror_harvests
        .entry(mirror.to_string())
        .or_default();
    match cell {
        Some(c) => {
            harvest_map.insert(epoch, c.to_vec());
        }
        None => {
            harvest_map.remove(&epoch);
        }
    }

    if epoch > window {
        return;
    }
    let Some(expected) = state.canonical.get(epoch) else {
        return;
    };
    let matches = cell.map(|c| c == expected).unwrap_or(false);
    if !matches {
        omit_de_whitelists_mirror(state, mirror, epoch);
    }
}

/// Publish epoch `e` with cell `c` into canonical log (establishes ground truth on receive path).
pub fn establish_canonical(state: &mut ValidForwardState, epoch: u64, cell: &[u8]) {
    if let Some(existing) = state.canonical.get(epoch) {
        if existing != cell {
            return;
        }
    }
    record_publish(&mut state.canonical, epoch, cell);
}

/// Mirror `m` correctly forwarded every published cell in window `[0, W]`.
pub fn valid_forward_party(state: &ValidForwardState, mirror: &str, window: u64) -> bool {
    if state.is_de_whitelisted(mirror) {
        return false;
    }
    for epoch in 0..=window {
        let Some(expected) = state.canonical.get(epoch) else {
            continue;
        };
        match state.harvest_view(mirror, epoch) {
            Some(got) if got == expected => {}
            _ => return false,
        }
    }
    true
}

/// Mirrors with ValidFwd and intact send rights (M_valid).
pub fn valid_mirror_set(state: &ValidForwardState, urls: &[String], window: u64) -> Vec<String> {
    urls.iter()
        .filter(|url| {
            valid_forward_party(state, url, window)
                && !availability_ledger::send_rights_revoked(state.mirror_actor(url))
        })
        .cloned()
        .collect()
}

/// ValidFwd over `[0, e-1]` required to harvest epoch `e` from mirror `m`.
pub fn receive_gate(state: &ValidForwardState, mirror: &str, epoch: u64) -> bool {
    for e in 0..epoch {
        let Some(expected) = state.canonical.get(e) else {
            continue;
        };
        match state.harvest_view(mirror, e) {
            Some(got) if got == expected => {}
            _ => return false,
        }
    }
    true
}

/// Selective omit breaks valid-forward history — de-whitelist mirror and record strike.
pub fn omit_de_whitelists_mirror(state: &mut ValidForwardState, mirror: &str, epoch: u64) {
    if state.de_whitelisted.insert(mirror.to_string()) {
        let actor = state.mirror_actor(mirror);
        availability_ledger::record_strike(actor, AvailabilityAttackKind::SelectiveOmit);
        eprintln!(
            "ValidForward: mirror {mirror} de-whitelisted at epoch {epoch} (selective omit)"
        );
    }
}

/// Mirror view mismatch vs canonical witness — record MirrorMismatch strike (once per mirror).
pub fn record_mirror_mismatch(state: &mut ValidForwardState, mirror: &str, epoch: u64) {
    if state.de_whitelisted.insert(mirror.to_string()) {
        let actor = state.mirror_actor(mirror);
        availability_ledger::record_strike(actor, AvailabilityAttackKind::MirrorMismatch);
        eprintln!(
            "ValidForward: mirror {mirror} de-whitelisted at epoch {epoch} (mirror mismatch)"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cell(tag: u8) -> Vec<u8> {
        vec![tag; 32]
    }

    #[test]
    fn valid_forward_omit_de_whitelists_mirror() {
        let mut state = ValidForwardState::new();
        state.register_mirror("mirror-a", 1001);
        state.register_mirror("mirror-b", 1002);
        let cell = sample_cell(5);
        record_publish(&mut state.canonical, 5, &cell);

        record_harvest(&mut state, "mirror-a", 5, None, 64);
        record_harvest(&mut state, "mirror-b", 5, Some(&cell), 64);

        assert!(!valid_forward_party(&state, "mirror-a", 64));
        assert!(valid_forward_party(&state, "mirror-b", 64));
        assert!(state.is_de_whitelisted("mirror-a"));
        assert!(!state.is_de_whitelisted("mirror-b"));
    }

    #[test]
    fn valid_forward_valid_mirror_set() {
        let mut state = ValidForwardState::new();
        let cell = sample_cell(7);
        record_publish(&mut state.canonical, 5, &cell);
        record_harvest(&mut state, "mirror-a", 5, None, 64);
        record_harvest(&mut state, "mirror-b", 5, Some(&cell), 64);

        let urls = vec!["mirror-a".to_string(), "mirror-b".to_string()];
        let valid = valid_mirror_set(&state, &urls, 64);
        assert_eq!(valid, vec!["mirror-b".to_string()]);
    }

    #[test]
    fn valid_forward_receive_gate_vacuous_at_zero() {
        let state = ValidForwardState::new();
        assert!(receive_gate(&state, "mirror-a", 0));
    }

    #[test]
    fn valid_forward_receive_gate_requires_prior_history() {
        let mut state = ValidForwardState::new();
        for e in 0..3u64 {
            let cell = sample_cell(e as u8);
            record_publish(&mut state.canonical, e, &cell);
            record_harvest(&mut state, "mirror-a", e, Some(&cell), 64);
        }
        assert!(receive_gate(&state, "mirror-a", 3));
        record_harvest(&mut state, "mirror-b", 1, None, 64);
        assert!(!receive_gate(&state, "mirror-b", 3));
    }
}
