#!/usr/bin/env python3
"""Minimal UES pool HTTP mirror for E2E and deploy reference."""
from __future__ import annotations

import argparse
import os
import struct
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse


class PoolMirrorHandler(BaseHTTPRequestHandler):
    store_dir: Path

    def do_GET(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path == "/pool/cells":
            qs = parse_qs(parsed.query)
            from_epoch = int(qs.get("from", ["0"])[0])
            body = self._harvest_body(from_epoch)
            self._ok(body, "application/octet-stream")
            return
        self.send_error(404)

    def do_POST(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path.startswith("/pool/cell"):
            qs = parse_qs(parsed.query)
            epoch = int(qs.get("epoch", ["0"])[0])
            length = int(self.headers.get("Content-Length", "0"))
            data = self.rfile.read(length)
            self.store_dir.mkdir(parents=True, exist_ok=True)
            out = self.store_dir / f"epoch_{epoch:08}.bin"
            out.write_bytes(data)
            self._ok(b"OK\n", "text/plain")
            return
        self.send_error(404)

    def _harvest_body(self, from_epoch: int) -> bytes:
        cells: list[tuple[int, bytes]] = []
        if not self.store_dir.is_dir():
            return b""
        for entry in sorted(self.store_dir.glob("epoch_*.bin")):
            try:
                epoch = int(entry.stem.split("_", 1)[1])
            except (IndexError, ValueError):
                continue
            if epoch >= from_epoch:
                cells.append((epoch, entry.read_bytes()))
        cells.sort(key=lambda x: x[0])
        buf = bytearray()
        for epoch, data in cells:
            buf.extend(struct.pack(">QI", epoch, len(data)))
            buf.extend(data)
        return bytes(buf)

    def _ok(self, body: bytes, content_type: str) -> None:
        self.send_response(200)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, fmt: str, *args) -> None:
        print(f"[pool-mirror] {self.address_string()} - {fmt % args}")


def main() -> None:
    p = argparse.ArgumentParser(description="UES pool HTTP mirror")
    p.add_argument("--host", default="127.0.0.1")
    p.add_argument("--port", type=int, default=9191)
    p.add_argument("--store-dir", default="./.pool-mirror-store")
    args = p.parse_args()
    store = Path(args.store_dir)
    store.mkdir(parents=True, exist_ok=True)
    PoolMirrorHandler.store_dir = store
    srv = HTTPServer((args.host, args.port), PoolMirrorHandler)
    print(f"pool mirror http://{args.host}:{args.port} store={store}")
    srv.serve_forever()


if __name__ == "__main__":
    main()
