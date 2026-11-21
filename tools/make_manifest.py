#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
make_manifest.py — собирает манифест ресурсов для ASTRA Launcher.

Формат, который ждёт лончер (`src-tauri/src/manifest.rs`):

    {
      "version": "1.0.0",
      "gta_version": "",
      "files": [{"path": "client_resources/index.js", "sha256": "...", "size": 81}]
    }

`path` — это одновременно и путь в URL (`{base_url}/{path}`), и путь, куда
лончер положит файл внутри каталога RAGE:MP. Поэтому каталог-источник и префикс
надо задавать явно.

Примеры
-------
    # разложить client_packages сервера в client_resources клиента
    python make_manifest.py --src D:\\redage_v3-main\\client_packages \\
        --prefix client_resources --version 1.0.0 \\
        --out D:\\redage_v3-main\\cdn\\manifest.json

    # только проверить, что попадёт в манифест
    python make_manifest.py --src .\\client_packages --prefix client_resources --dry-run
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

#: Что не тащим в клиент.
SKIP_DIRS = {".git", "node_modules", "__pycache__", ".vscode", "dist", "build"}
SKIP_SUFFIXES = {".pdb", ".map", ".log", ".tmp"}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def collect(src: Path, prefix: str, include: tuple[str, ...], exclude: tuple[str, ...]):
    entries = []
    for path in sorted(src.rglob("*")):
        if not path.is_file():
            continue
        rel = path.relative_to(src)
        if any(part in SKIP_DIRS for part in rel.parts):
            continue
        if path.suffix.lower() in SKIP_SUFFIXES:
            continue
        if include and path.suffix.lower() not in include:
            continue
        if exclude and any(part in rel.parts for part in exclude):
            continue
        url_path = "/".join(part for part in (prefix, rel.as_posix()) if part)
        entries.append({
            "path": url_path,
            "sha256": sha256(path),
            "size": path.stat().st_size,
        })
    return entries


def main() -> int:
    parser = argparse.ArgumentParser(description="Манифест ресурсов для ASTRA Launcher")
    parser.add_argument("--src", required=True, help="каталог с ресурсами сервера")
    parser.add_argument("--prefix", default="client_resources",
                        help="префикс пути внутри клиента (по умолчанию client_resources)")
    parser.add_argument("--version", default="1.0.0")
    parser.add_argument("--gta-version", default="")
    parser.add_argument("--out", help="куда записать manifest.json")
    parser.add_argument("--include", default="",
                        help="только эти расширения, через запятую: .js,.json,.html")
    parser.add_argument("--exclude", default="",
                        help="пропустить каталоги, через запятую")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    src = Path(args.src).expanduser().resolve()
    if not src.is_dir():
        print(f"! нет каталога: {src}")
        return 2

    include = tuple(x.strip().lower() for x in args.include.split(",") if x.strip())
    exclude = tuple(x.strip() for x in args.exclude.split(",") if x.strip())

    files = collect(src, args.prefix.strip("/"), include, exclude)
    total = sum(item["size"] for item in files)
    print(f"источник: {src}")
    print(f"файлов: {len(files)}, суммарно {total / 1048576:.2f} МиБ")
    for item in files[:10]:
        print(f"  {item['path']:<52} {item['size']:>9,} Б")
    if len(files) > 10:
        print(f"  … ещё {len(files) - 10}")

    manifest = {
        "version": args.version,
        "gta_version": args.gta_version,
        "files": files,
    }

    if args.dry_run:
        print("\n--dry-run: файл не записан")
        return 0

    out = Path(args.out).expanduser().resolve() if args.out else src / "manifest.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(manifest, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"\nманифест: {out} ({out.stat().st_size:,} Б)")
    print(f"base_url должен указывать на каталог, внутри которого лежит '{args.prefix}'")
    return 0


if __name__ == "__main__":
    sys.exit(main())
