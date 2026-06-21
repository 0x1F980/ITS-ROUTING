//! CoverTransport — L3' constant harvest: pool cells + benign E-channel cover every epoch.

use std::io;
use std::thread;
use std::time::{Duration, Instant};

use crate::aeh::fetch_live_entropy;
use crate::courier::build_epoch_courier;

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
    entropy_sources: Vec<String>,
}

impl PoolPlusCoverHarvest {
    pub fn new(
        pool_file: &str,
        pool_url: &str,
        multi_pool_urls: &[String],
        entropy_sources: &[String],
    ) -> Self {
        PoolPlusCoverHarvest {
            pool_file: pool_file.to_string(),
            pool_url: pool_url.to_string(),
            multi_pool_urls: multi_pool_urls.to_vec(),
            entropy_sources: entropy_sources.to_vec(),
        }
    }

    /// Harvest pool cells from `from_epoch` and fetch all configured E-sources (cover).
    pub fn harvest_epoch(&self, from_epoch: u64) -> io::Result<CoverEpochBundle> {
        let courier = build_epoch_courier(
            &self.pool_file,
            &self.pool_url,
            &self.multi_pool_urls,
        );
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
