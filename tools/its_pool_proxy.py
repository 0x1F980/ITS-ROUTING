#!/usr/bin/env python3
"""Minimal SOCKS5/HTTP bridge → ITS wire → UES pool (v1.8)."""
from __future__ import annotations

import argparse
import socket
import struct
import subprocess
import tempfile
from pathlib import Path


def socks5_handshake(conn: socket.socket) -> tuple[str, int] | None:
    data = conn.recv(256)
    if len(data) < 2 or data[0] != 0x05:
        return None
    conn.sendall(b"\x05\x00")
    req = conn.recv(256)
    if len(req) < 7 or req[1] != 0x01:
        return None
    atyp = req[3]
    if atyp == 0x01:
        host = socket.inet_ntoa(req[4:8])
        port = struct.unpack("!H", req[8:10])[0]
    elif atyp == 0x03:
        ln = req[4]
        host = req[5 : 5 + ln].decode()
        port = struct.unpack("!H", req[5 + ln : 7 + ln])[0]
    else:
        return None
    conn.sendall(b"\x05\x00\x00\x01\x00\x00\x00\x00\x00\x00")
    return host, port


def handle_client(
    conn: socket.socket,
    routing: Path,
    config: Path,
    ratchet: Path,
    pk: Path,
    asym: Path,
) -> None:
    target = socks5_handshake(conn)
    if not target:
        conn.close()
        return
    host, port = target
    payload = f"GET / HTTP/1.1\r\nHost: {host}:{port}\r\n\r\n".encode()
    with tempfile.TemporaryDirectory() as td:
        td_path = Path(td)
        plain = td_path / "req.bin"
        wire = td_path / "msg.wire"
        recv_wire = td_path / "recv.wire"
        plain.write_bytes(payload)
        subprocess.run(
            [str(asym), "encrypt", "--pk", str(pk), "--in", str(plain), "--out", str(wire)],
            check=True,
        )
        subprocess.run(
            [
                str(routing),
                "-c",
                str(config),
                "client-send",
                "--pool",
                "-f",
                str(wire),
                "-d",
                "1",
                "--ratchet-seed-file",
                str(ratchet),
            ],
            check=True,
        )
        subprocess.run(
            [
                str(routing),
                "-c",
                str(config),
                "client-receive",
                "--pool",
                "--continuous",
                "--timeout-secs",
                "10",
                "-o",
                str(recv_wire),
                "--ratchet-seed-file",
                str(ratchet),
            ],
            check=True,
        )
        if recv_wire.exists():
            conn.sendall(recv_wire.read_bytes()[:512])
    conn.close()


def main() -> None:
    p = argparse.ArgumentParser(description="ITS pool SOCKS5 proxy")
    p.add_argument("--listen", default="127.0.0.1:1080")
    p.add_argument("--config", required=True)
    p.add_argument("--ratchet-seed-file", required=True)
    p.add_argument("--pk", required=True)
    p.add_argument("--routing", default="its-routing")
    p.add_argument("--asymmetric", default="its_asymmetric")
    args = p.parse_args()
    host, port_s = args.listen.rsplit(":", 1)
    port = int(port_s)
    routing = Path(args.routing)
    asym = Path(args.asymmetric)
    config = Path(args.config)
    ratchet = Path(args.ratchet_seed_file)
    pk = Path(args.pk)
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind((host, port))
    srv.listen(8)
    print(f"its-pool-proxy SOCKS5 {host}:{port}")
    while True:
        conn, _ = srv.accept()
        try:
            handle_client(conn, routing, config, ratchet, pk, asym)
        except Exception as exc:
            print(f"proxy error: {exc}")
            conn.close()


if __name__ == "__main__":
    main()
