//! ITS pool SOCKS5 proxy — encrypt client payload → UES pool → decrypt peer reply.
//!
//! Orchestrates `its_asymmetric` + `its-routing` via subprocess (production path).
//! Optional KM orchestration: set `ITS_KM_BIN` and pass `--use-km` (documented in
//! ITS-routing_SOCKS_EGRESS.md); default is direct asymmetric + routing CLIs.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use clap::Parser;
use tempfile::TempDir;

const MAX_PAYLOAD: usize = 4 * 1024 * 1024;
const CLIENT_READ_IDLE: Duration = Duration::from_millis(2000);

#[derive(Parser, Debug)]
#[command(name = "its-pool-proxy", about = "ITS pool SOCKS5 proxy (production)")]
struct Args {
    /// SOCKS listen address (host:port)
    #[arg(long, default_value = "127.0.0.1:1080")]
    listen: String,

    /// its-routing config (routing.toml)
    #[arg(long)]
    config: PathBuf,

    /// Shared transport ratchet seed file (OOB with peer)
    #[arg(long)]
    ratchet_seed_file: PathBuf,

    /// Peer public key (Bob) — encrypt outbound payloads
    #[arg(long)]
    pk: PathBuf,

    /// Local secret key (Alice) — decrypt inbound pool replies
    #[arg(long)]
    sk: PathBuf,

    /// Local public key (Alice) — paired with --sk for decrypt
    #[arg(long)]
    own_pk: PathBuf,

    /// its-routing binary (overrides ITS_ROUTING_BIN)
    #[arg(long)]
    routing: Option<PathBuf>,

    /// its_asymmetric binary (overrides ITS_ASYMMETRIC_BIN)
    #[arg(long)]
    asymmetric: Option<PathBuf>,

    /// client-receive timeout seconds per SOCKS session
    #[arg(long, default_value_t = 30)]
    receive_timeout_secs: u64,

    /// Milliseconds to wait after publish before harvesting peer reply (avoids concurrent receive)
    #[arg(long, default_value_t = 2500)]
    reply_grace_ms: u64,
}

struct Binaries {
    routing: PathBuf,
    asymmetric: PathBuf,
}

impl Binaries {
    fn from_args(args: &Args) -> Self {
        Self {
            routing: args.routing.clone().unwrap_or_else(|| {
                std::env::var("ITS_ROUTING_BIN")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("its-routing"))
            }),
            asymmetric: args.asymmetric.clone().unwrap_or_else(|| {
                std::env::var("ITS_ASYMMETRIC_BIN")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("its_asymmetric"))
            }),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let bins = Binaries::from_args(&args);
    let listen: SocketAddr = args.listen.parse()?;

    let listener = TcpListener::bind(listen)?;
    eprintln!("its-pool-proxy SOCKS5 {listen}");

    for stream in listener.incoming() {
        match stream {
            Ok(conn) => {
                if let Err(e) = handle_client(conn, &args, &bins) {
                    eprintln!("proxy error: {e}");
                }
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
    Ok(())
}

fn handle_client(mut conn: TcpStream, args: &Args, bins: &Binaries) -> Result<(), Box<dyn std::error::Error>> {
    let target = match socks5_handshake(&mut conn)? {
        Some(t) => t,
        None => return Ok(()),
    };

    let mut payload = read_client_payload(&mut conn)?;
    if payload.is_empty() {
        let (host, port) = &target;
        payload = format!("GET / HTTP/1.1\r\nHost: {host}:{port}\r\n\r\n").into_bytes();
    }
    if payload.is_empty() {
        return Ok(());
    }

    let work = TempDir::new()?;
    let plain = work.path().join("req.bin");
    let wire = work.path().join("msg.wire");
    let recv_wire = work.path().join("recv.wire");
    let resp_plain = work.path().join("resp.bin");

    std::fs::write(&plain, &payload)?;

    run_check(
        Command::new(&bins.asymmetric)
            .arg("encrypt")
            .arg("--pk")
            .arg(&args.pk)
            .arg("--in")
            .arg(&plain)
            .arg("--out")
            .arg(&wire),
    )?;

    run_check(
        Command::new(&bins.routing)
            .arg("-c")
            .arg(&args.config)
            .arg("client-send")
            .arg("--pool")
            .arg("-f")
            .arg(&wire)
            .arg("--ratchet-seed-file")
            .arg(&args.ratchet_seed_file),
    )?;

    if let Ok(marker) = std::env::var("ITS_PROXY_SENT_MARKER") {
        let _ = std::fs::write(marker, b"sent");
    }

    wait_for_reply_marker(args.reply_grace_ms);

    let response = receive_peer_response(
        &args.config,
        &args.ratchet_seed_file,
        &args.sk,
        &args.own_pk,
        args.receive_timeout_secs,
        &recv_wire,
        &resp_plain,
        bins,
    )?;

    if response.is_empty() {
        return Ok(());
    }
    conn.write_all(&response)?;
    Ok(())
}

/// Poll pool until decrypted payload looks like a peer reply (not our echoed request).
fn wait_for_reply_marker(fallback_ms: u64) {
    let deadline = std::time::Instant::now() + Duration::from_millis(fallback_ms.max(5000));
    if let Ok(reply_marker) = std::env::var("ITS_PROXY_REPLY_MARKER") {
        while std::time::Instant::now() < deadline {
            if std::path::Path::new(&reply_marker).exists() {
                std::thread::sleep(Duration::from_millis(300));
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
    if !remaining.is_zero() {
        std::thread::sleep(remaining);
    }
}

fn receive_peer_response(
    config: &PathBuf,
    ratchet: &PathBuf,
    sk: &PathBuf,
    own_pk: &PathBuf,
    timeout_secs: u64,
    recv_wire: &PathBuf,
    resp_plain: &PathBuf,
    bins: &Binaries,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
    let mut attempts = 0u32;
    while std::time::Instant::now() < deadline && attempts < 12 {
        attempts += 1;
        let _ = std::fs::remove_file(recv_wire);
        let _ = std::fs::remove_file(resp_plain);

        let remaining = deadline
            .saturating_duration_since(std::time::Instant::now())
            .as_secs()
            .max(5)
            .min(30);

        let status = Command::new(&bins.routing)
            .arg("-c")
            .arg(config)
            .arg("client-receive")
            .arg("--pool")
            .arg("--continuous")
            .arg("--timeout-secs")
            .arg(remaining.to_string())
            .arg("-o")
            .arg(recv_wire)
            .arg("--ratchet-seed-file")
            .arg(ratchet)
            .status()?;

        if !status.success() || !recv_wire.exists() {
            continue;
        }

        if run_check(
            Command::new(&bins.asymmetric)
                .arg("decrypt")
                .arg("--sk")
                .arg(sk)
                .arg("--pk")
                .arg(own_pk)
                .arg("--in")
                .arg(recv_wire)
                .arg("--out")
                .arg(resp_plain),
        )
        .is_err()
        {
            continue;
        }

        let plain = std::fs::read(resp_plain)?;
        if plain.is_empty() || looks_like_outbound_request(&plain) {
            continue;
        }
        return Ok(plain);
    }
    Err("timed out waiting for peer response on pool".into())
}

fn looks_like_outbound_request(plain: &[u8]) -> bool {
    let s = plain.strip_prefix(b"\xef\xbb\xbf").unwrap_or(plain);
    s.starts_with(b"GET ")
        || s.starts_with(b"POST ")
        || s.starts_with(b"HEAD ")
        || s.starts_with(b"PUT ")
        || s.starts_with(b"DELETE ")
        || s.starts_with(b"OPTIONS ")
        || s.starts_with(b"PATCH ")
        || s.starts_with(b"CONNECT ")
}

fn run_check(cmd: &mut Command) -> Result<(), Box<dyn std::error::Error>> {
    let out = cmd.output()?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        return Err(format!(
            "{} failed ({}): {stdout}{stderr}",
            cmd.get_program().to_string_lossy(),
            out.status
        )
        .into());
    }
    Ok(())
}

/// SOCKS5 handshake + CONNECT; returns bound target after sending success reply.
fn socks5_handshake(conn: &mut TcpStream) -> Result<Option<(String, u16)>, Box<dyn std::error::Error>> {
    let mut buf = [0u8; 256];
    let n = conn.read(&mut buf)?;
    if n < 2 || buf[0] != 0x05 {
        return Ok(None);
    }
    conn.write_all(&[0x05, 0x00])?;

    let n = conn.read(&mut buf)?;
    if n < 7 || buf[1] != 0x01 {
        return Ok(None);
    }

    let (host, port) = parse_socks5_target(&buf[..n])?;
    conn.write_all(&[0x05, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
    Ok(Some((host, port)))
}

fn parse_socks5_target(req: &[u8]) -> Result<(String, u16), Box<dyn std::error::Error>> {
    if req.len() < 7 {
        return Err("SOCKS request too short".into());
    }
    match req[3] {
        0x01 => {
            if req.len() < 10 {
                return Err("SOCKS IPv4 request too short".into());
            }
            let host = format!("{}.{}.{}.{}", req[4], req[5], req[6], req[7]);
            let port = u16::from_be_bytes([req[8], req[9]]);
            Ok((host, port))
        }
        0x03 => {
            let ln = req[4] as usize;
            if req.len() < 5 + ln + 2 {
                return Err("SOCKS domain request too short".into());
            }
            let host = String::from_utf8(req[5..5 + ln].to_vec())?;
            let port = u16::from_be_bytes([req[5 + ln], req[5 + ln + 1]]);
            Ok((host, port))
        }
        _ => Err("unsupported SOCKS ATYP".into()),
    }
}

fn read_client_payload(conn: &mut TcpStream) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    conn.set_read_timeout(Some(CLIENT_READ_IDLE))?;
    let mut out = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        match conn.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                out.extend_from_slice(&buf[..n]);
                if out.len() > MAX_PAYLOAD {
                    return Err("client payload exceeds limit".into());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }
    conn.set_read_timeout(None)?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_domain_target() {
        let req = b"\x05\x01\x00\x03\x0bexample.com\x00\x50";
        let (host, port) = parse_socks5_target(req).unwrap();
        assert_eq!(host, "example.com");
        assert_eq!(port, 80);
    }
}
