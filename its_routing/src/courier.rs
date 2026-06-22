use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::valid_forward_party::{
    self, establish_canonical, record_harvest, record_publish, receive_gate, valid_mirror_set,
    ValidForwardState,
};
use crate::witness_consensus::consensus_at_epoch;

use zeroize::{Zeroize, ZeroizeOnDrop};

/// A memory-secured container that zeroizes its contents upon drop to protect RAM state.
#[derive(Zeroize, ZeroizeOnDrop, Default)]
pub(crate) struct ZeroizedBuffer {
    pub(crate) data: Vec<u8>,
}

impl ZeroizedBuffer {
    pub(crate) fn new(data: Vec<u8>) -> Self {
        ZeroizedBuffer { data }
    }
}

// ==============================================================================
// PACKET COURIER ABSTRACTION (FOR TRANSPORT-PROTOCOL AGNOSTICISM)
// ==============================================================================

/// An abstract transport-layer courier that can receive and dispatch raw packets.
pub(crate) trait PacketCourier {
    fn send_raw(&self, data: &[u8], addr: &str) -> std::io::Result<()>;
    fn recv_raw(&self, buf: &mut [u8]) -> std::io::Result<(usize, String)>;
}

/// A standard UDP socket implementation of the `PacketCourier` trait.
pub(crate) struct UdpCourier {
    socket: std::net::UdpSocket,
}

impl UdpCourier {
    pub(crate) fn new(socket: std::net::UdpSocket) -> Self {
        UdpCourier { socket }
    }
}

impl PacketCourier for UdpCourier {
    fn send_raw(&self, data: &[u8], addr: &str) -> std::io::Result<()> {
        if let Ok(socket_addr) = addr.parse::<std::net::SocketAddr>() {
            self.socket.send_to(data, socket_addr)?;
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid address format",
            ))
        }
    }

    fn recv_raw(&self, buf: &mut [u8]) -> std::io::Result<(usize, String)> {
        let (len, src) = self.socket.recv_from(buf)?;
        Ok((len, src.to_string()))
    }
}

// ==============================================================================
// UES POOL EPOCH COURIER (Fase 2 — Monocell Pool)
// ==============================================================================

/// Publish and harvest fixed-size epoch cells in the UES Monocell Pool.
pub trait EpochCourier {
    fn publish_cell(&self, epoch: u64, cell: &[u8]) -> std::io::Result<()>;
    fn harvest_cells(&self, from_epoch: u64) -> std::io::Result<Vec<(u64, Vec<u8>)>>;
}

/// File-backed pool: writes `epoch_NNNNNNNN.bin` in a directory.
pub struct FilePoolCourier {
    dir: PathBuf,
}

impl FilePoolCourier {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        FilePoolCourier {
            dir: dir.as_ref().to_path_buf(),
        }
    }

    fn cell_path(&self, epoch: u64) -> PathBuf {
        self.dir.join(format!("epoch_{:08}.bin", epoch))
    }
}

impl EpochCourier for FilePoolCourier {
    fn publish_cell(&self, epoch: u64, cell: &[u8]) -> std::io::Result<()> {
        fs::create_dir_all(&self.dir)?;
        fs::write(self.cell_path(epoch), cell)
    }

    fn harvest_cells(&self, from_epoch: u64) -> std::io::Result<Vec<(u64, Vec<u8>)>> {
        if !self.dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in fs::read_dir(&self.dir)? {
            let entry = entry?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("epoch_") || !name.ends_with(".bin") {
                continue;
            }
            let num_str = &name[6..name.len() - 4];
            let epoch: u64 = match num_str.parse() {
                Ok(v) => v,
                Err(_) => continue,
            };
            if epoch < from_epoch {
                continue;
            }
            let data = fs::read(entry.path())?;
            out.push((epoch, data));
        }
        out.sort_by_key(|(e, _)| *e);
        Ok(out)
    }
}

/// HTTP pool courier — wraps a file backend or posts to a local HTTP stub.
pub struct HttpPoolCourier {
    base_url: String,
    file_backend: FilePoolCourier,
}

impl HttpPoolCourier {
    pub fn new(base_url: impl Into<String>, file_dir: impl AsRef<Path>) -> Self {
        HttpPoolCourier {
            base_url: base_url.into(),
            file_backend: FilePoolCourier::new(file_dir),
        }
    }

    fn use_file_only(&self) -> bool {
        self.base_url.is_empty()
            || self.base_url.starts_with("file://")
            || self.base_url.starts_with('/')
    }
}

fn prod_gate_active() -> bool {
    std::env::var("ITS_PROD_GATE").ok().as_deref() == Some("1")
}

impl EpochCourier for HttpPoolCourier {
    fn publish_cell(&self, epoch: u64, cell: &[u8]) -> std::io::Result<()> {
        if self.use_file_only() {
            return self.file_backend.publish_cell(epoch, cell);
        }
        match http_post_cell(&self.base_url, epoch, cell) {
            Ok(()) => Ok(()),
            Err(e) => {
                if prod_gate_active() {
                    Err(e)
                } else {
                    eprintln!("HttpPoolCourier: HTTP publish failed ({e}); falling back to file pool.");
                    self.file_backend.publish_cell(epoch, cell)
                }
            }
        }
    }

    fn harvest_cells(&self, from_epoch: u64) -> std::io::Result<Vec<(u64, Vec<u8>)>> {
        if self.use_file_only() {
            return self.file_backend.harvest_cells(from_epoch);
        }
        match http_harvest_cells(&self.base_url, from_epoch) {
            Ok(cells) if !cells.is_empty() => Ok(cells),
            Ok(_) if prod_gate_active() => Ok(Vec::new()),
            Ok(_) => Ok(Vec::new()),
            Err(e) if prod_gate_active() => Err(e),
            Err(_) => self.file_backend.harvest_cells(from_epoch),
        }
    }
}

/// Publish the same cell to multiple pool couriers (multi-path A-resilience).
/// Operational alternate path: harvest from next mirror in M_valid (Lean: ValidForwardParty).
pub struct MultiCourier {
    couriers: Vec<Box<dyn EpochCourier + Send + Sync>>,
}

impl MultiCourier {
    pub fn new(couriers: Vec<Box<dyn EpochCourier + Send + Sync>>) -> Self {
        MultiCourier { couriers }
    }
}

impl EpochCourier for MultiCourier {
    fn publish_cell(&self, epoch: u64, cell: &[u8]) -> std::io::Result<()> {
        let mut last_err = None;
        for c in &self.couriers {
            match c.publish_cell(epoch, cell) {
                Ok(()) => {}
                Err(e) => last_err = Some(e),
            }
        }
        if let Some(e) = last_err {
            Err(e)
        } else {
            Ok(())
        }
    }

    fn harvest_cells(&self, from_epoch: u64) -> std::io::Result<Vec<(u64, Vec<u8>)>> {
        let mut merged: Vec<(u64, Vec<u8>)> = Vec::new();
        for c in &self.couriers {
            if let Ok(cells) = c.harvest_cells(from_epoch) {
                for (epoch, data) in cells {
                    if !merged.iter().any(|(e, _)| *e == epoch) {
                        merged.push((epoch, data));
                    }
                }
            }
        }
        merged.sort_by_key(|(e, _)| *e);
        Ok(merged)
    }
}

/// Multi-mirror courier with ValidFwd tracking, M_valid filter, and optional witness consensus.
pub struct WhitelistMultiCourier {
    mirror_urls: Vec<String>,
    couriers: Vec<Box<dyn EpochCourier + Send + Sync>>,
    valid_fwd: Arc<Mutex<ValidForwardState>>,
    witness_pool_urls: Vec<String>,
    consensus_k: usize,
    valid_fwd_window: u64,
}

impl WhitelistMultiCourier {
    pub fn new(
        mirror_urls: Vec<String>,
        couriers: Vec<Box<dyn EpochCourier + Send + Sync>>,
        valid_fwd: Arc<Mutex<ValidForwardState>>,
        witness_pool_urls: Vec<String>,
        consensus_k: usize,
        valid_fwd_window: u64,
    ) -> Self {
        if let Ok(mut state) = valid_fwd.lock() {
            for (idx, url) in mirror_urls.iter().enumerate() {
                state.register_mirror(url, valid_forward_party::stable_mirror_actor(url) + idx as u32);
            }
        }
        WhitelistMultiCourier {
            mirror_urls,
            couriers,
            valid_fwd,
            witness_pool_urls,
            consensus_k: consensus_k.max(1),
            valid_fwd_window,
        }
    }
}

impl EpochCourier for WhitelistMultiCourier {
    fn publish_cell(&self, epoch: u64, cell: &[u8]) -> std::io::Result<()> {
        if let Ok(mut state) = self.valid_fwd.lock() {
            record_publish(&mut state.canonical, epoch, cell);
        }
        let mut last_err = None;
        for c in &self.couriers {
            match c.publish_cell(epoch, cell) {
                Ok(()) => {}
                Err(e) => last_err = Some(e),
            }
        }
        if let Some(e) = last_err {
            Err(e)
        } else {
            Ok(())
        }
    }

    fn harvest_cells(&self, from_epoch: u64) -> std::io::Result<Vec<(u64, Vec<u8>)>> {
        let mut per_mirror: Vec<(String, Vec<(u64, Vec<u8>)>)> = Vec::new();
        for (url, courier) in self.mirror_urls.iter().zip(self.couriers.iter()) {
            let cells = courier.harvest_cells(from_epoch).unwrap_or_default();
            per_mirror.push((url.clone(), cells));
        }

        let mut state = self
            .valid_fwd
            .lock()
            .map_err(|_| std::io::Error::other("ValidForwardState lock poisoned"))?;

        let window = self.valid_fwd_window;
        let mut epochs_seen: std::collections::HashSet<u64> = state.canonical.published_epochs().collect();
        for (_, cells) in &per_mirror {
            for (epoch, _) in cells {
                epochs_seen.insert(*epoch);
            }
        }

        for epoch in epochs_seen.iter().copied().filter(|e| *e >= from_epoch) {
            if state.canonical.get(epoch).is_none() {
                let candidates: Vec<(String, Vec<u8>)> = per_mirror
                    .iter()
                    .filter_map(|(url, cells)| {
                        cells
                            .iter()
                            .find(|(e, _)| *e == epoch)
                            .map(|(_, c)| (url.clone(), c.clone()))
                    })
                    .collect();
                if candidates.is_empty() {
                    continue;
                }
                let witness_harvests: Vec<(String, Option<Vec<u8>>)> = candidates
                    .iter()
                    .filter(|(url, _)| {
                        self.witness_pool_urls.is_empty()
                            || self.witness_pool_urls.iter().any(|w| w == url)
                    })
                    .map(|(url, cell)| (url.clone(), Some(cell.clone())))
                    .collect();
                if !self.witness_pool_urls.is_empty() {
                    if let Some((_, cell)) = witness_harvests.first() {
                        if let Some(cell) = cell {
                            if consensus_at_epoch(
                                &witness_harvests,
                                epoch,
                                cell,
                                self.consensus_k,
                            ) {
                                establish_canonical(&mut state, epoch, cell);
                                continue;
                            }
                        }
                    }
                }
                if let Some((_, cell)) = candidates.first() {
                    establish_canonical(&mut state, epoch, cell);
                }
            }
        }

        let canonical_epochs: Vec<u64> = state
            .canonical
            .published_epochs()
            .filter(|e| *e >= from_epoch)
            .collect();

        for (url, cells) in &per_mirror {
            let mut harvested_epochs: std::collections::HashSet<u64> =
                cells.iter().map(|(e, _)| *e).collect();
            for (epoch, cell) in cells {
                record_harvest(&mut state, url, *epoch, Some(cell), window);
                harvested_epochs.insert(*epoch);
            }
            for epoch in &canonical_epochs {
                if harvested_epochs.contains(epoch) {
                    continue;
                }
                if state.canonical.get(*epoch).is_some() && state.harvest_view(url, *epoch).is_none()
                {
                    record_harvest(&mut state, url, *epoch, None, window);
                }
            }
        }

        let valid = valid_mirror_set(&state, &self.mirror_urls, window);
        let mut merged: Vec<(u64, Vec<u8>)> = Vec::new();

        for epoch in canonical_epochs {
            let Some(expected) = state.canonical.get(epoch).map(|c| c.to_vec()) else {
                continue;
            };
            if !self.witness_pool_urls.is_empty() {
                let witness_harvests: Vec<(String, Option<Vec<u8>>)> = valid
                    .iter()
                    .filter(|url| self.witness_pool_urls.iter().any(|w| w == *url))
                    .map(|url| {
                        (
                            url.clone(),
                            state.harvest_view(url, epoch).map(|c| c.to_vec()),
                        )
                    })
                    .collect();
                if !witness_harvests.is_empty()
                    && !consensus_at_epoch(&witness_harvests, epoch, &expected, self.consensus_k)
                {
                    continue;
                }
            }
            let mut chosen: Option<Vec<u8>> = None;
            for url in &valid {
                if !receive_gate(&state, url, epoch) {
                    continue;
                }
                if let Some(got) = state.harvest_view(url, epoch) {
                    if got == expected.as_slice() {
                        chosen = Some(got.to_vec());
                        break;
                    }
                }
            }
            if chosen.is_none() {
                for url in &valid {
                    if let Some(got) = state.harvest_view(url, epoch) {
                        if got == expected.as_slice() {
                            chosen = Some(got.to_vec());
                            break;
                        }
                    }
                }
            }
            if let Some(cell) = chosen {
                merged.push((epoch, cell));
            }
        }

        if merged.is_empty() {
            for url in &valid {
                if let Some((url, cells)) = per_mirror.iter().find(|(u, _)| u == url) {
                    let _ = url;
                    for (epoch, cell) in cells {
                        if *epoch >= from_epoch && !merged.iter().any(|(e, _)| *e == *epoch) {
                            merged.push((*epoch, cell.clone()));
                        }
                    }
                }
            }
        }

        merged.sort_by_key(|(e, _)| *e);
        Ok(merged)
    }
}

/// Parameters for epoch courier construction (shared ValidForwardState).
pub struct EpochCourierBuild<'a> {
    pub pool_file: &'a str,
    pub pool_url: &'a str,
    pub multi_pool_urls: &'a [String],
    pub witness_pool_urls: &'a [String],
    pub consensus_k: usize,
    pub valid_fwd_window: u64,
    pub valid_fwd_state: Arc<Mutex<ValidForwardState>>,
}

impl<'a> EpochCourierBuild<'a> {
    pub fn build(self) -> Box<dyn EpochCourier + Send + Sync> {
        build_epoch_courier_from(self)
    }
}

/// Build the configured epoch courier from pool settings.
pub fn build_epoch_courier(
    pool_file: &str,
    pool_url: &str,
    multi_pool_urls: &[String],
) -> Box<dyn EpochCourier + Send + Sync> {
    build_epoch_courier_from(EpochCourierBuild {
        pool_file,
        pool_url,
        multi_pool_urls,
        witness_pool_urls: &[],
        consensus_k: 1,
        valid_fwd_window: 64,
        valid_fwd_state: Arc::new(Mutex::new(ValidForwardState::new())),
    })
}

pub fn build_epoch_courier_from(args: EpochCourierBuild<'_>) -> Box<dyn EpochCourier + Send + Sync> {
    let mirror_urls = mirror_url_list(args.pool_url, args.multi_pool_urls);
    if mirror_urls.len() > 1 {
        let couriers: Vec<Box<dyn EpochCourier + Send + Sync>> = mirror_urls
            .iter()
            .map(|url| {
                Box::new(HttpPoolCourier::new(url.clone(), args.pool_file))
                    as Box<dyn EpochCourier + Send + Sync>
            })
            .collect();
        return Box::new(WhitelistMultiCourier::new(
            mirror_urls,
            couriers,
            args.valid_fwd_state,
            args.witness_pool_urls.to_vec(),
            args.consensus_k,
            args.valid_fwd_window,
        ));
    }
    if args.pool_url.is_empty() {
        Box::new(FilePoolCourier::new(args.pool_file))
    } else {
        Box::new(HttpPoolCourier::new(args.pool_url, args.pool_file))
    }
}

fn mirror_url_list(pool_url: &str, multi_pool_urls: &[String]) -> Vec<String> {
    let mut urls = Vec::new();
    if !pool_url.is_empty() {
        urls.push(pool_url.to_string());
    }
    for u in multi_pool_urls {
        if !u.is_empty() && !urls.iter().any(|x| x == u) {
            urls.push(u.clone());
        }
    }
    urls
}

fn http_post_cell(base_url: &str, epoch: u64, cell: &[u8]) -> std::io::Result<()> {
    let path = format!("/pool/cell?epoch={epoch}");
    let (host, port) = parse_http_host(base_url)?;
    let mut stream = TcpStream::connect((host.as_str(), port))?;
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        cell.len()
    );
    stream.write_all(req.as_bytes())?;
    stream.write_all(cell)?;
    let mut resp = Vec::new();
    stream.read_to_end(&mut resp)?;
    if resp.windows(12).any(|w| w.starts_with(b"HTTP/1.1 200") || w.starts_with(b"HTTP/1.0 200")) {
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "HTTP POST did not return 200",
        ))
    }
}

fn http_harvest_cells(base_url: &str, from_epoch: u64) -> std::io::Result<Vec<(u64, Vec<u8>)>> {
    let path = format!("/pool/cells?from={from_epoch}");
    let (host, port) = parse_http_host(base_url)?;
    let mut stream = TcpStream::connect((host.as_str(), port))?;
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(req.as_bytes())?;
    let mut resp = Vec::new();
    stream.read_to_end(&mut resp)?;
    let body = http_body(&resp);
    parse_harvest_body(body)
}

fn parse_http_host(base_url: &str) -> std::io::Result<(String, u16)> {
    let stripped = base_url
        .trim()
        .strip_prefix("http://")
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "need http:// URL"))?;
    let (host, port) = match stripped.split_once(':') {
        Some((h, p)) => (h.to_string(), p.parse::<u16>().unwrap_or(80)),
        None => (stripped.to_string(), 80),
    };
    Ok((host, port))
}

fn http_body(resp: &[u8]) -> &[u8] {
    resp.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|i| &resp[i + 4..])
        .unwrap_or(resp)
}

fn parse_harvest_body(body: &[u8]) -> std::io::Result<Vec<(u64, Vec<u8>)>> {
    let mut out = Vec::new();
    let mut offset = 0;
    while offset + 12 <= body.len() {
        let epoch = u64::from_be_bytes(body[offset..offset + 8].try_into().unwrap());
        let len = u32::from_be_bytes(body[offset + 8..offset + 12].try_into().unwrap()) as usize;
        offset += 12;
        if offset + len > body.len() {
            break;
        }
        out.push((epoch, body[offset..offset + len].to_vec()));
        offset += len;
    }
    Ok(out)
}
