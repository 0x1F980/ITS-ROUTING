//! Availability ledger — strike counter and send-rights revocation (Lean: `AvailabilityLedger.lean`).
//!
//! Operational contract: after `DEFAULT_STRIKE_THRESHOLD` disclosed availability attacks,
//! pool epoch publish is forbidden for the actor. Full persistence lives in ITS-ledger
//! (`AvailabilityStrikeStore`); this module is the in-process enforcement gate.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Default strike threshold N (matches `defaultAvailabilityStrikeThreshold` in Lean).
pub const DEFAULT_STRIKE_THRESHOLD: u32 = 3;

/// Attack tags aligned with `AvailabilityAttackKind` in `AvailabilityLedger.lean`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AvailabilityAttackKind {
    SelectiveOmit,
    MirrorMismatch,
    RateDeltaGap,
    SssDeletionExceedsBound,
}

impl AvailabilityAttackKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelectiveOmit => "selective_omit",
            Self::MirrorMismatch => "mirror_mismatch",
            Self::RateDeltaGap => "rate_delta_gap",
            Self::SssDeletionExceedsBound => "sss_deletion_exceeds_bound",
        }
    }
}

/// In-process ledger state (mirrors `AvailabilityLedgerState`).
#[derive(Debug, Clone)]
pub struct AvailabilityLedger {
    strike_threshold: u32,
    strikes: HashMap<u32, u32>,
}

impl Default for AvailabilityLedger {
    fn default() -> Self {
        Self::new(DEFAULT_STRIKE_THRESHOLD)
    }
}

impl AvailabilityLedger {
    pub fn new(strike_threshold: u32) -> Self {
        Self {
            strike_threshold: strike_threshold.max(1),
            strikes: HashMap::new(),
        }
    }

    pub fn strike_threshold(&self) -> u32 {
        self.strike_threshold
    }

    pub fn strike_count(&self, actor: u32) -> u32 {
        self.strikes.get(&actor).copied().unwrap_or(0)
    }

    /// Record one slash event for a disclosed availability attack.
    pub fn record_strike(&mut self, actor: u32, kind: AvailabilityAttackKind) {
        let entry = self.strikes.entry(actor).or_insert(0);
        *entry = entry.saturating_add(1);
        eprintln!(
            "Availability ledger: strike {} for actor {actor} ({})",
            self.strike_count(actor),
            kind.as_str()
        );
    }

    /// `sendRightsRevoked` — strikes ≥ N.
    pub fn send_rights_revoked(&self, actor: u32) -> bool {
        self.strike_count(actor) >= self.strike_threshold
    }

    /// `poolPublishAllowed` — publish only below threshold.
    pub fn pool_publish_allowed(&self, actor: u32) -> bool {
        !self.send_rights_revoked(actor)
    }
}

static GLOBAL_LEDGER: OnceLock<Mutex<AvailabilityLedger>> = OnceLock::new();

fn global_ledger() -> &'static Mutex<AvailabilityLedger> {
    GLOBAL_LEDGER.get_or_init(|| Mutex::new(AvailabilityLedger::default()))
}

pub fn strike_count(actor: u32) -> u32 {
    global_ledger()
        .lock()
        .map(|l| l.strike_count(actor))
        .unwrap_or(0)
}

pub fn record_strike(actor: u32, kind: AvailabilityAttackKind) {
    if let Ok(mut ledger) = global_ledger().lock() {
        ledger.record_strike(actor, kind);
    }
}

pub fn send_rights_revoked(actor: u32) -> bool {
    global_ledger()
        .lock()
        .map(|l| l.send_rights_revoked(actor))
        .unwrap_or(false)
}

pub fn pool_publish_allowed(actor: u32) -> bool {
    global_ledger()
        .lock()
        .map(|l| l.pool_publish_allowed(actor))
        .unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slash_threshold_revokes_send_rights() {
        let mut ledger = AvailabilityLedger::new(3);
        let actor = 42;
        assert!(ledger.pool_publish_allowed(actor));
        ledger.record_strike(actor, AvailabilityAttackKind::RateDeltaGap);
        ledger.record_strike(actor, AvailabilityAttackKind::MirrorMismatch);
        assert!(ledger.pool_publish_allowed(actor));
        ledger.record_strike(actor, AvailabilityAttackKind::SelectiveOmit);
        assert!(ledger.send_rights_revoked(actor));
        assert!(!ledger.pool_publish_allowed(actor));
    }
}
