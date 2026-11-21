#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
serve_local.py — локальный CDN для проверки ASTRA Launcher без хостинга.

Отдаёт каталог по HTTP с поддержкой Range (лончер качает файлы потоком и
умеет докачивать), печатает каждый запрос — сразу видно, что и откуда тянется.

    python serve_local.py --root D:\\redage_v3-main --port 8787

В конфиге лончера (%APPDATA%\\astra-launcher\\config.json):

    "client_manifest_url":    "http://127.0.0.1:8787/cdn/manifest.json",
    "file_download_base_url": "http://127.0.0.1:8787",
    "server_address":         "127.0.0.1:22005"

Манифест при этом должен содержать пути вида `client_resources/...`,
а каталог `client_packages` лежать в корне раздачи — тогда
`{base_url}/{path}` попадёт в нужный файл.
"""

from __future__ import annotations

import argparse
import http.server
import os
import socketserver
import sys
from pathlib import Path


class RangeHandler(http.server.SimpleHTTPRequestHandler):
    """SimpleHTTPRequestHandler + Range + понятный лог."""

    server_version = "AstraLocalCDN/1.0"

    def __init__(self, *args, directory: str | None = None, **kwargs) -> None:
        super().__init__(*args, directory=directory, **kwargs)

    def end_headers(self) -> None:
        self.send_header("Accept-Ranges", "bytes")
        self.send_header("Cache-Control", "no-store")
        self.send_header("Access-Control-Allow-Origin", "*")
        super().end_headers()

    def log_message(self, fmt: str, *args) -> None:
        sys.stderr.write(f"  {self.address_string()} {fmt % args}\n")

    def do_GET(self) -> None:  # noqa: N802
        path = self.translate_path(self.path)
        if os.path.isdir(path):
            return super().do_GET()

        try:
            handle = open(path, "rb")
        except OSError:
            self.send_error(404, "not found")
            return

        with handle:
            size = os.fstat(handle.fileno()).st_size
            start, end = 0, size - 1
            status = 200
            header = self.headers.get("Range")
            if header and header.startswith("bytes="):
                spec = header[6:].split(",")[0].strip()
                first, _, last = spec.partition("-")
                if first.isdigit():
                    start = int(first)
                if last.isdigit():
                    end = min(int(last), size - 1)
                if start > end or start >= size:
                    self.send_response(416)
                    self.send_header("Content-Range", f"bytes */{size}")
                    self.end_headers()
                    return
                status = 206

            length = end - start + 1
            self.send_response(status)
            self.send_header("Content-Type", self.guess_type(path))
            self.send_header("Content-Length", str(length))
            if status == 206:
                self.send_header("Content-Range", f"bytes {start}-{end}/{size}")
            self.end_headers()

            handle.seek(start)
            remaining = length
            while remaining > 0:
                chunk = handle.read(min(256 * 1024, remaining))
                if not chunk:
                    break
                try:
                    self.wfile.write(chunk)
                except (BrokenPipeError, ConnectionResetError):
                    return
                remaining -= len(chunk)


def main() -> int:
    parser = argparse.ArgumentParser(description="Локальный CDN для лончера")
    parser.add_argument("--root", required=True, help="каталог раздачи")
    parser.add_argument("--port", type=int, default=8787)
    parser.add_argument("--bind", default="127.0.0.1")
    parser.add_argument("--strip", default="",
                        help="срезать префикс из URL, например client_resources")
    args = parser.parse_args()

    root = Path(args.root).expanduser().resolve()
    if not root.is_dir():
        print(f"! нет каталога: {root}")
        return 2

    strip = args.strip.strip("/")

    class Handler(RangeHandler):
        def translate_path(self, path: str) -> str:
            if strip and path.startswith("/" + strip + "/"):
                path = path[len(strip) + 1:]
            return super().translate_path(path)

    handler = lambda *a, **kw: Handler(*a, directory=str(root), **kw)  # noqa: E731
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.ThreadingTCPServer((args.bind, args.port), handler) as server:
        print(f"CDN: {root}")
        print(f"URL: http://{args.bind}:{args.port}/  (Ctrl+C для остановки)")
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            print("\nостановлено")
    return 0


if __name__ == "__main__":
    sys.exit(main())
