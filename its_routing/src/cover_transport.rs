//! CoverTransport — L3' constant harvest: pool cells + benign E-channel cover every epoch.

use std::io;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::aeh::fetch_live_entropy;
use crate::courier::{build_epoch_courier_from, EpochCourierBuild};
use crate::valid_forward_party::ValidForwardState;

/// One epoch harvest bundle: pool cells plus cover-traffic byte count (O⁺ symmetry).
pub struct CoverEpochBundle {
    pub pool_cells: Vec<(u64, Vec<u8>)>,
    pub cover_bytes: usize,
}

/// Multiplex pool courier with constant E-channel cover harvest.
pub struct PoolPlusCoverHarvest {
    pool_file: String,
    pool_url: String,
    multi_pool_urls: Vec<String>,
    witness_pool_urls: Vec<String>,
    consensus_k: usize,
    valid_fwd_window: u64,
    entropy_sources: Vec<String>,
    valid_fwd_state: Arc<Mutex<ValidForwardState>>,
}

impl PoolPlusCoverHarvest {
    pub fn new(
        pool_file: &str,
        pool_url: &str,
        multi_pool_urls: &[String],
        witness_pool_urls: &[String],
        consensus_k: usize,
        valid_fwd_window: u64,
        entropy_sources: &[String],
        valid_fwd_state: Arc<Mutex<ValidForwardState>>,
    ) -> Self {
        PoolPlusCoverHarvest {
            pool_file: pool_file.to_string(),
            pool_url: pool_url.to_string(),
            multi_pool_urls: multi_pool_urls.to_vec(),
            witness_pool_urls: witness_pool_urls.to_vec(),
            consensus_k,
            valid_fwd_window,
            entropy_sources: entropy_sources.to_vec(),
            valid_fwd_state,
        }
    }

    /// Harvest pool cells from `from_epoch` and fetch all configured E-sources (cover).
    pub fn harvest_epoch(&self, from_epoch: u64) -> io::Result<CoverEpochBundle> {
        let courier = build_epoch_courier_from(EpochCourierBuild {
            pool_file: &self.pool_file,
            pool_url: &self.pool_url,
            multi_pool_urls: &self.multi_pool_urls,
            witness_pool_urls: &self.witness_pool_urls,
            consensus_k: self.consensus_k,
            valid_fwd_window: self.valid_fwd_window,
            valid_fwd_state: Arc::clone(&self.valid_fwd_state),
        });
        let pool_cells = courier.harvest_cells(from_epoch)?;
        let cover = fetch_live_entropy(&self.entropy_sources);
        Ok(CoverEpochBundle {
            pool_cells,
            cover_bytes: cover.len(),
        })
    }
}

/// Wall-clock epoch ticker for L3' constant-rate polling.
pub struct EpochLoop {
    interval: Duration,
    started: Instant,
}

impl EpochLoop {
    pub fn new(interval_ms: u64) -> Self {
        EpochLoop {
            interval: Duration::from_millis(interval_ms.max(1)),
            started: Instant::now(),
        }
    }

    pub fn wait_tick(&self) {
        thread::sleep(self.interval);
    }

    pub fn elapsed_secs(&self) -> u64 {
        self.started.elapsed().as_secs()
    }

    pub fn timed_out(&self, timeout_secs: u64) -> bool {
        timeout_secs > 0 && self.elapsed_secs() >= timeout_secs
    }
}
