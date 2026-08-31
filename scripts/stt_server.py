#!/usr/bin/env python3
"""OpenAI-compatible STT wrapper around whisper-cli (exhibit / LAN use).

POST /v1/audio/transcriptions  ->  {"text": "..."}
GET  /health                   ->  {"ok": true}

Does not call cloud APIs. Bind defaults to 127.0.0.1; pass --host for the booth LAN.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Dict, Optional, Tuple


def parse_multipart(content_type: str, body: bytes) -> Tuple[Dict[str, bytes], Dict[str, str]]:
    bound = None
    for part in content_type.split(";"):
        part = part.strip()
        if part.lower().startswith("boundary="):
            bound = part.split("=", 1)[1].strip().strip('"')
            break
    if not bound:
        raise ValueError("multipart boundary missing")
    delim = b"--" + bound.encode("ascii", "replace")
    files: Dict[str, bytes] = {}
    fields: Dict[str, str] = {}
    for chunk in body.split(delim):
        chunk = chunk.strip(b"\r\n")
        if not chunk or chunk == b"--" or chunk.startswith(b"--"):
            continue
        header_blob, sep, payload = chunk.partition(b"\r\n\r\n")
        if not sep:
            continue
        name = None
        filename = None
        for line in header_blob.decode("utf-8", "replace").split("\r\n"):
            if not line.lower().startswith("content-disposition:"):
                continue
            for item in line.split(";")[1:]:
                item = item.strip()
                if item.startswith("name="):
                    name = item.split("=", 1)[1].strip('"')
                elif item.startswith("filename="):
                    filename = item.split("=", 1)[1].strip('"')
        if not name:
            continue
        if filename is not None:
            files[name] = payload
        else:
            fields[name] = payload.decode("utf-8", "replace")
    return files, fields


def run_whisper(
    cli: str,
    model: Path,
    wav_path: Path,
    language: str,
    threads: int,
) -> str:
    prefix = wav_path.with_suffix("")
    cmd = [
        cli,
        "-m",
        str(model),
        "-f",
        str(wav_path),
        "-l",
        language,
        "-t",
        str(threads),
        "-nt",
        "-np",
        "-otxt",
        "-of",
        str(prefix),
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
    txt_path = Path(str(prefix) + ".txt")
    if proc.returncode != 0 and not txt_path.is_file():
        err = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(err or f"whisper-cli exit {proc.returncode}")
    if txt_path.is_file():
        text = txt_path.read_text(encoding="utf-8", errors="replace")
    else:
        text = proc.stdout or ""
    return " ".join(text.split())


class Handler(BaseHTTPRequestHandler):
    server_version = "strudel-stt/1.0"

    def log_message(self, fmt: str, *args) -> None:
        sys.stderr.write("%s - %s\n" % (self.address_string(), fmt % args))

    def _json(self, code: int, payload: dict) -> None:
        raw = json.dumps(payload, ensure_ascii=False).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def _auth_ok(self) -> bool:
        token = self.server.token  # type: ignore[attr-defined]
        if not token:
            return True
        got = self.headers.get("Authorization", "")
        return got == f"Bearer {token}"

    def do_GET(self) -> None:  # noqa: N802
        if self.path.split("?", 1)[0] == "/health":
            self._json(200, {"ok": True})
            return
        self._json(404, {"error": "not found"})

    def do_POST(self) -> None:  # noqa: N802
        path = self.path.split("?", 1)[0]
        if path != "/v1/audio/transcriptions":
            self._json(404, {"error": "not found"})
            return
        if not self._auth_ok():
            self._json(401, {"error": "unauthorized"})
            return
        length = int(self.headers.get("Content-Length", "0") or "0")
        if length <= 0 or length > 8 * 1024 * 1024:
            self._json(400, {"error": "invalid body"})
            return
        body = self.rfile.read(length)
        ctype = self.headers.get("Content-Type", "")
        try:
            files, fields = parse_multipart(ctype, body)
        except ValueError as e:
            self._json(400, {"error": str(e)})
            return
        wav = files.get("file")
        if not wav:
            self._json(400, {"error": "missing file"})
            return
        language = fields.get("language") or self.server.language  # type: ignore[attr-defined]
        with tempfile.TemporaryDirectory(prefix="strudel-stt-") as tmp:
            wav_path = Path(tmp) / "audio.wav"
            wav_path.write_bytes(wav)
            try:
                text = run_whisper(
                    self.server.whisper_cli,  # type: ignore[attr-defined]
                    self.server.model,  # type: ignore[attr-defined]
                    wav_path,
                    language,
                    self.server.threads,  # type: ignore[attr-defined]
                )
            except FileNotFoundError:
                self._json(500, {"error": "whisper-cli not found"})
                return
            except RuntimeError as e:
                self._json(500, {"error": str(e)[:300]})
                return
        self._json(200, {"text": text})


def main() -> int:
    p = argparse.ArgumentParser(description="Local OpenAI-shaped STT for strudel-rs F12")
    p.add_argument("--host", default="127.0.0.1", help="bind address (default 127.0.0.1)")
    p.add_argument("--port", type=int, default=8090)
    p.add_argument("--model", required=True, help="path to ggml Whisper model")
    p.add_argument("--whisper-cli", default="whisper-cli", help="whisper-cli binary")
    p.add_argument("--threads", type=int, default=2, help="whisper threads (leave CPU for Qwen)")
    p.add_argument("--language", default="ja")
    p.add_argument("--token", default="", help="optional Bearer token")
    args = p.parse_args()

    model = Path(args.model).expanduser()
    if not model.is_file():
        print(f"model not found: {model}", file=sys.stderr)
        return 1
    cli = args.whisper_cli
    if os.path.sep in cli or (os.path.altsep and os.path.altsep in cli):
        if not Path(cli).exists():
            print(f"whisper-cli not found: {cli}", file=sys.stderr)
            return 1
    elif not shutil.which(cli):
        print(f"whisper-cli not on PATH: {cli}", file=sys.stderr)
        return 1

    httpd = ThreadingHTTPServer((args.host, args.port), Handler)
    httpd.model = model  # type: ignore[attr-defined]
    httpd.whisper_cli = cli  # type: ignore[attr-defined]
    httpd.threads = max(1, args.threads)  # type: ignore[attr-defined]
    httpd.language = args.language  # type: ignore[attr-defined]
    httpd.token = args.token  # type: ignore[attr-defined]
    print(f"stt on http://{args.host}:{args.port}/v1/audio/transcriptions", flush=True)
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("bye", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
