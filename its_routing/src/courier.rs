use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

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
            Err(e) if prod_gate_active() => Err(e),
            Ok(_) | Err(_) => self.file_backend.harvest_cells(from_epoch),
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

/// Build the configured epoch courier from pool settings.
pub fn build_epoch_courier(
    pool_file: &str,
    pool_url: &str,
    multi_pool_urls: &[String],
) -> Box<dyn EpochCourier + Send + Sync> {
    if !multi_pool_urls.is_empty() {
        let couriers: Vec<Box<dyn EpochCourier + Send + Sync>> = multi_pool_urls
            .iter()
            .map(|url| {
                Box::new(HttpPoolCourier::new(url.clone(), pool_file)) as Box<dyn EpochCourier + Send + Sync>
            })
            .collect();
        return Box::new(MultiCourier::new(couriers));
    }
    if pool_url.is_empty() {
        Box::new(FilePoolCourier::new(pool_file))
    } else {
        Box::new(HttpPoolCourier::new(pool_url, pool_file))
    }
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
