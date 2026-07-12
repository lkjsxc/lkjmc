#!/usr/bin/env python3
"""Probe daemon configuration only against an explicitly disposable database."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import tempfile
import time
from urllib.parse import unquote, urlsplit


def port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def disposable(parsed) -> bool:
    name = parsed.path.removeprefix("/")
    compose = os.environ.get("LKJMC_E_CONTROL_COMPOSE") == "1"
    host = parsed.hostname in {"localhost", "127.0.0.1", "::1"}
    host = host or (compose and parsed.hostname == "postgres")
    return (
        os.environ.get("LKJMC_LAB_POSTGRES_DISPOSABLE") == "1"
        and parsed.scheme in {"postgres", "postgresql"}
        and host and not parsed.query and not parsed.fragment
        and bool(parsed.username and parsed.password)
        and bool(re.fullmatch(r"lkjmc_lab_[a-z0-9_]+", name))
    )


def config(root: Path, parsed, http_port: int, token: Path, pool: int, memory: int, changed: bool) -> dict:
    config_root = root / ("config-next" if changed else "config-current")
    for path in (config_root, root / "data", root / "logs", root / "jars", root / "assets"):
        path.mkdir(exist_ok=True)
    return {
        "installRoot": str(root), "configRoot": str(config_root),
        "dataRoot": str(root / "data"), "logRoot": str(root / "logs"),
        "socketPath": str(root / "daemon.sock"),
        "database": {"host": parsed.hostname, "port": parsed.port or 5432,
                     "database": parsed.path[1:], "user": unquote(parsed.username),
                     "secretFile": str(root / "database.secret"), "poolSize": pool},
        "network": {"name": "lkjmc-lab", "defaultLocale": "en", "fallbackServer": "hub",
                    "onlineMode": True, "velocityForwarding": "modern",
                    "forwardingSecretFile": str(root / "forwarding.secret"),
                    "javaEntry": {"bindHost": "127.0.0.1", "port": 25565,
                                  "publicHosts": [], "preferredPublicHost": None},
                    "bedrockEntry": {"mode": "auto", "host": "127.0.0.1", "port": 19132}},
        "jars": {"root": str(root / "jars"), "defaultChannel": "stable",
                 "userAgent": "lkjmc (+https://github.com/lkjsxc/lkjmc)"},
        "daemonHttp": {"enabled": True, "address": f"127.0.0.1:{http_port}", "tokenFile": str(token)},
        "assets": {"root": str(root / "assets"), "serverChannel": "stable", "pluginChannel": "stable",
                   "userAgent": "lkjmc (+https://github.com/lkjsxc/lkjmc)", "downloadTimeoutSeconds": 120},
        "plugins": {"lkjmc": {"enabled": True}, "viaversion": {"mode": "auto", "installOn": "backend"},
                    "viabackwards": {"mode": "auto", "installOn": "backend"},
                    "geyser": {"mode": "auto", "installOn": "proxy"},
                    "floodgate": {"mode": "auto", "installOn": "proxy", "backendApi": False}},
        "runtime": {"adapter": "local-process", "defaultJavaMemoryMb": memory,
                    "proxyJavaMemoryMb": 512, "stopTimeoutSeconds": 30,
                    "portRangeStart": 25566, "portRangeEnd": 25665},
    }


def command(values: list[str], env: dict[str, str], required: bool = True) -> str:
    done = subprocess.run(values, env=env, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
                          text=True, timeout=60, check=False)
    if required and done.returncode:
        raise RuntimeError("command-failed")
    return done.stdout if done.returncode == 0 else ""


def fails(values: list[str], env: dict[str, str]) -> bool:
    return subprocess.run(values, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                          timeout=60, check=False).returncode != 0


def wait_database(cli: str, env: dict[str, str]) -> None:
    for _ in range(120):
        done = subprocess.run([cli, "db", "migrate"], env=env, stdout=subprocess.DEVNULL,
                              stderr=subprocess.DEVNULL, timeout=60, check=False)
        if done.returncode == 0:
            return
        time.sleep(0.05)
    raise RuntimeError("database-health-timeout")


def http_ok(http_port: int, token: str) -> bool:
    body = b'{"requestId":"config-probe","actor":{"kind":"cli","name":"lab"},"command":"doctor","body":{}}'
    request = b"POST /command HTTP/1.1\r\nHost: lab\r\nContent-Type: application/json\r\nAuthorization: Bearer " + token.encode() + b"\r\nContent-Length: " + str(len(body)).encode() + b"\r\nConnection: close\r\n\r\n" + body
    try:
        with socket.create_connection(("127.0.0.1", http_port), timeout=1) as connection:
            connection.sendall(request)
            return connection.recv(32).startswith(b"HTTP/1.1 200")
    except OSError:
        return False


def wait_ready(process: subprocess.Popen, socket_path: Path, http_port: int, token: str) -> None:
    for _ in range(120):
        if process.poll() is not None:
            raise RuntimeError("daemon-exited-before-ready")
        if socket_path.exists() and http_ok(http_port, token):
            return
        time.sleep(0.05)
    raise RuntimeError("daemon-not-ready")


def status(cli: str, socket_path: Path, env: dict[str, str]) -> dict:
    return json.loads(command([cli, "--json", "--socket", str(socket_path), "status"], env))


def terminate(process: subprocess.Popen) -> None:
    process.terminate()
    try:
        process.wait(10)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait(10)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--daemon", required=True)
    parser.add_argument("--cli", required=True)
    args = parser.parse_args()
    parsed = urlsplit(os.environ.get("LKJMC_LAB_POSTGRES_URL", ""))
    if not disposable(parsed):
        print("E-CONTROL config result=BLOCKED reason=disposable-loopback-or-compose-database-required")
        return 1
    env, process = os.environ.copy(), None
    try:
        with tempfile.TemporaryDirectory(prefix="lkjmc-e-control-config-") as temporary:
            root = Path(temporary)
            (root / "database.secret").write_text(unquote(parsed.password), encoding="utf-8")
            (root / "forwarding.secret").write_text("lab-forwarding", encoding="utf-8")
            first, second = port(), port()
            token_one, token_two = root / "token-one", root / "token-two"
            token_one.write_text("config-token-one", encoding="utf-8")
            token_two.write_text("config-token-two", encoding="utf-8")
            env["LKJMC_DATABASE_URL"] = os.environ["LKJMC_LAB_POSTGRES_URL"]
            wait_database(args.cli, env)
            config_path = root / "lkjmc.json"
            config_path.write_text(json.dumps(config(root, parsed, first, token_one, 1, 1024, False)), encoding="utf-8")
            process = subprocess.Popen([args.daemon, "--config", str(config_path)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            wait_ready(process, root / "daemon.sock", first, "config-token-one")
            config_path.write_text(json.dumps(config(root, parsed, second, token_two, 2, 1536, True)), encoding="utf-8")
            command([args.cli, "--socket", str(root / "daemon.sock"), "config", "reload"], env)
            reloaded = status(args.cli, root / "daemon.sock", env)
            partial = (reloaded["database"]["poolSize"] == 2 and reloaded["roots"]["config"].endswith("config-next")
                       and reloaded["http"]["address"].endswith(f":{first}") and http_ok(first, "config-token-one")
                       and not http_ok(second, "config-token-two"))
            invalid = config(root, parsed, second, token_two, 2, 0, True)
            config_path.write_text(json.dumps(invalid), encoding="utf-8")
            runtime_validated = fails([args.cli, "--socket", str(root / "daemon.sock"), "config", "reload"], env)
            config_path.write_text(json.dumps(config(root, parsed, second, token_two, 2, 1536, True)), encoding="utf-8")
            if not partial or not runtime_validated:
                raise RuntimeError("reload-observation-missing")
            terminate(process)
            process = subprocess.Popen([args.daemon, "--config", str(config_path)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            wait_ready(process, root / "daemon.sock", second, "config-token-two")
            restarted = status(args.cli, root / "daemon.sock", env)
            if restarted["database"]["poolSize"] != 2 or http_ok(first, "config-token-one"):
                raise RuntimeError("restart-application-observation-missing")
            print("E-CONTROL config=reload result=REJECTED pool-and-roots=applied listener-and-credential=retained runtime=validated-not-observable")
            print("E-CONTROL config=restart-required result=PASS pool-listener-credential=applied runtime=validated-at-start")
    except (KeyError, OSError, RuntimeError, ValueError, subprocess.SubprocessError):
        print("E-CONTROL config result=BLOCKED reason=bounded-config-attempt-failed")
        return 1
    finally:
        if process is not None and process.poll() is None:
            terminate(process)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
