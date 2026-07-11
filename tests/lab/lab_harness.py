#!/usr/bin/env python3
"""Small real-boundary helpers; this module contains no product adapters."""
from __future__ import annotations

import os
from pathlib import Path
import re
import secrets
import shutil
import socket
import subprocess
import tempfile
from typing import Iterable

from lab_redaction import redact_text

ARTIFACT_BYTES = 8192
ARTIFACT_RUNS = 20
ROOT = Path(__file__).resolve().parents[2]


class Lab:
    _allocated_ports: set[int] = set()

    def __init__(self, label: str) -> None:
        token = secrets.token_hex(6)
        safe = re.sub(r"[^a-z0-9-]", "-", label.lower())
        self.name = f"lkjmc-lab-{safe}-{token}"
        self.root = Path(tempfile.mkdtemp(prefix=f"{self.name}-"))
        parent = Path(os.environ.get("LKJMC_LAB_ARTIFACT_ROOT", tempfile.gettempdir()))
        self.artifacts = parent / "lkjmc-lab-artifacts" / self.name
        self.artifacts.mkdir(parents=True, exist_ok=True)
        self.schema = f"lkjmc_lab_{token}"
        self.compose_project = self.name.replace("-", "_")
        self.secrets: set[str] = set()
        self.ports: set[int] = set()
        self.children: list[tuple[str, subprocess.Popen[bytes], object]] = []
        self.listeners: list[tuple[socket.socket, Path | None]] = []
        self.compose_started = False
        self.closed = False
        self._prune_artifacts()

    def port(self) -> int:
        while True:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
                listener.bind(("127.0.0.1", 0))
                port = int(listener.getsockname()[1])
            if port not in Lab._allocated_ports:
                Lab._allocated_ports.add(port)
                self.ports.add(port)
                return port

    def listen_tcp(self) -> socket.socket:
        listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        listener.bind(("127.0.0.1", 0))
        listener.listen(1)
        port = int(listener.getsockname()[1])
        Lab._allocated_ports.add(port)
        self.ports.add(port)
        self.listeners.append((listener, None))
        return listener

    def listen_unix(self, name: str) -> Path:
        path = self.path(name)
        listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        listener.bind(str(path))
        listener.listen(1)
        self.listeners.append((listener, path))
        return path

    def path(self, name: str) -> Path:
        if Path(name).name != name:
            raise ValueError("lab paths must stay under the isolated root")
        return self.root / name

    def redact(self, text: str) -> str:
        return redact_text(text, self.secrets)

    def record(self, name: str, text: str | bytes) -> Path:
        if Path(name).name != name:
            raise ValueError("artifact names must be plain filenames")
        value = text.decode("utf-8", "replace") if isinstance(text, bytes) else text
        path = self.artifacts / name
        path.write_text(self.redact(value)[-ARTIFACT_BYTES:], encoding="utf-8")
        return path

    def start(self, label: str, args: Iterable[str], env: dict[str, str] | None = None) -> subprocess.Popen[bytes]:
        stream = self.path(f"{label}.raw").open("wb")
        try:
            process = subprocess.Popen(list(args), cwd=ROOT, env=os.environ | (env or {}), stdout=stream, stderr=subprocess.STDOUT)
        except BaseException:
            stream.close()
            raise
        self.children.append((label, process, stream))
        return process

    def run(self, label: str, args: Iterable[str], timeout: int, env: dict[str, str] | None = None) -> tuple[int, str]:
        process = self.start(label, args, env)
        try:
            process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()
            raise RuntimeError(f"{label} timed out")
        self._capture(process)
        raw = self.path(f"{label}.raw")
        return process.returncode, raw.read_text(encoding="utf-8", errors="replace")

    def compose(self, args: Iterable[str], timeout: int = 240) -> tuple[int, str]:
        docker = shutil.which("docker")
        if docker is None:
            raise FileNotFoundError("docker is unavailable")
        values = list(args)
        if "up" in values:
            self.compose_started = True
        command = [docker, "compose", "--project-name", self.compose_project, "-f", str(ROOT / "docker-compose.yml"), *values]
        return self.run("compose-" + (values[0] if values else "run"), command, timeout)

    def close(self) -> None:
        if self.closed:
            return
        try:
            self._close_listeners()
            for _, process, _ in self.children:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()
                self._capture(process)
        finally:
            try:
                if self.compose_started:
                    self.compose(["down", "--volumes", "--remove-orphans"], 120)
            except (OSError, RuntimeError):
                self.record("compose-cleanup.log", "compose cleanup could not start")
            finally:
                shutil.rmtree(self.root, ignore_errors=True)
                Lab._allocated_ports.difference_update(self.ports)
                self.closed = True

    def _close_listeners(self) -> None:
        for listener, path in self.listeners:
            listener.close()
            if path is not None:
                path.unlink(missing_ok=True)

    def _capture(self, process: subprocess.Popen[bytes]) -> None:
        for label, child, stream in self.children:
            if child is process:
                stream.close()
                raw = self.path(f"{label}.raw")
                self.record(f"{label}.log", raw.read_bytes() if raw.exists() else b"")
                return

    def _prune_artifacts(self) -> None:
        old = sorted(self.artifacts.parent.glob("lkjmc-lab-*"), key=lambda path: path.stat().st_mtime, reverse=True)
        for path in old[ARTIFACT_RUNS:]:
            if path.is_dir():
                shutil.rmtree(path, ignore_errors=True)


def http_post_tcp(port: int, body: str, token: str | None = None) -> tuple[int, str]:
    return _http_post(socket.create_connection(("127.0.0.1", port), timeout=2), body, token)


def http_post_unix(path: Path, body: str, token: str | None = None) -> tuple[int, str]:
    connection = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    connection.settimeout(2)
    connection.connect(str(path))
    return _http_post(connection, body, token)


def _http_post(connection: socket.socket, body: str, token: str | None) -> tuple[int, str]:
    data = body.encode("utf-8")
    auth = f"Authorization: Bearer {token}\r\n" if token else ""
    request = ("POST /command HTTP/1.1\r\nHost: lkjmc-lab\r\nContent-Type: application/json\r\n"
               f"Content-Length: {len(data)}\r\n{auth}Connection: close\r\n\r\n").encode() + data
    with connection:
        connection.sendall(request)
        raw = b""
        while b"\r\n\r\n" not in raw:
            raw += connection.recv(4096)
        head, body = raw.split(b"\r\n\r\n", 1)
        lines = head.decode("iso-8859-1").split("\r\n")
        length = next((int(line.split(":", 1)[1]) for line in lines[1:] if line.lower().startswith("content-length:")), 0)
        while len(body) < length:
            body += connection.recv(4096)
        return int(lines[0].split()[1]), body[:length].decode("utf-8", "replace")
