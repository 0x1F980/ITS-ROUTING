#!/usr/bin/env python3
"""Minimal ITS wire HTTP proxy (Eco D) — POST /its/wire stores raw .wire bytes."""
from __future__ import annotations

import argparse
import os
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path


class ItsWireHandler(BaseHTTPRequestHandler):
    out_dir: Path

    def do_POST(self) -> None:
        if self.path not in ("/its/wire", "/its/wire/"):
            self.send_error(404, "use POST /its/wire")
            return
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length)
        alpn = self.headers.get("ALPN", self.headers.get("X-ALPN", "its-wire/1-compact"))
        out = self.out_dir / f"wire_{len(body)}.bin"
        out.write_bytes(body)
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.end_headers()
        msg = f"stored {len(body)} bytes alpn={alpn} -> {out}\n"
        self.wfile.write(msg.encode())
        print(msg, end="")

    def log_message(self, fmt: str, *args) -> None:
        print(f"[its_wire_proxy] {self.address_string()} - {fmt % args}")


def main() -> None:
    p = argparse.ArgumentParser(description="ITS wire HTTP proxy")
    p.add_argument("--host", default="127.0.0.1")
    p.add_argument("--port", type=int, default=8765)
    p.add_argument("--out-dir", default="./.its-wire-inbox")
    args = p.parse_args()
    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    ItsWireHandler.out_dir = out_dir
    srv = HTTPServer((args.host, args.port), ItsWireHandler)
    print(f"ITS wire proxy listening http://{args.host}:{args.port}/its/wire -> {out_dir}")
    srv.serve_forever()


if __name__ == "__main__":
    main()
