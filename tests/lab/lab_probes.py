#!/usr/bin/env python3
"""Run F-LAB probes against real local or explicitly disposable boundaries."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import socket
import sys
import time
import uuid

sys.dont_write_bytecode = True
from lab_boundaries import java_client_real, postgres_real
from lab_harness import Lab, ROOT, http_post_tcp, http_post_unix
from lab_outcomes import Blocked, Skip


def daemon_http_real(lab: Lab) -> None:
    daemon = ROOT / "target/debug/lkjmc-daemon"
    if not daemon.is_file() or not os.access(daemon, os.X_OK):
        raise Skip("target/debug/lkjmc-daemon is not built")
    port, token, socket_path = lab.port(), uuid.uuid4().hex, lab.path("daemon.sock")
    lab.secrets.add(token)
    process = lab.start("daemon", [str(daemon), "--socket", str(socket_path), "--http", f"127.0.0.1:{port}", "--http-token", token, "--config-root", str(lab.path("config")), "--log-root", str(lab.path("logs")), "--jar-root", str(lab.path("jars")), "--data-root", str(lab.path("data"))])
    for _ in range(100):
        if process.poll() is not None:
            raise Blocked("daemon exited before its listeners became ready")
        if socket_path.exists():
            try:
                _success(http_post_tcp(port, _request("doctor"), token), "daemon TCP HTTP")
                _success(http_post_unix(socket_path, _request("doctor")), "daemon Unix HTTP")
                return
            except (OSError, ValueError, json.JSONDecodeError):
                pass
        time.sleep(0.05)
    raise Blocked("daemon listeners did not become ready")


def process_real(lab: Lab) -> None:
    child = lab.start("process", [sys.executable, "-u", "-c", "import time; print('process-real-ready'); time.sleep(60)"])
    time.sleep(0.1)
    if child.poll() is not None:
        raise Blocked("local child exited before observation")
    lab.record("process-observation.txt", "local child process remained alive; cleanup terminates it")


def isolation_cleanup(lab: Lab) -> None:
    listener = lab.listen_tcp()
    port = int(listener.getsockname()[1])
    socket_path = lab.listen_unix("held.sock")
    child = lab.start("held-process", [sys.executable, "-u", "-c", "import time; time.sleep(60)"])
    _connect_tcp(port)
    _connect_unix(socket_path)
    lab.cleanup_proof = (port, socket_path, child)
    if os.environ.get("LKJMC_LAB_COMPOSE") == "1":
        _compose_real(lab)
    if os.environ.get("LKJMC_LAB_PROTOCOL") == "1":
        _protocol_player(lab)


def cleanup_proved(lab: Lab) -> None:
    port, socket_path, child = lab.cleanup_proof
    if lab.root.exists() or socket_path.exists() or child.poll() is None:
        raise Blocked("laboratory cleanup left a root, socket, or process")
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", port))
    socket_path.parent.mkdir()
    unix = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    try:
        unix.bind(str(socket_path))
    finally:
        unix.close()
        socket_path.unlink(missing_ok=True)
        socket_path.parent.rmdir()


def _connect_tcp(port: int) -> None:
    with socket.create_connection(("127.0.0.1", port), timeout=2):
        pass


def _connect_unix(path: Path) -> None:
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
        connection.settimeout(2)
        connection.connect(str(path))


def _request(command: str) -> str:
    return json.dumps({"requestId": str(uuid.uuid4()), "actor": {"kind": "cli", "name": "lab"}, "command": command, "body": {}})


def _success(response: tuple[int, str], boundary: str) -> None:
    status, body = response
    if status != 200 or not json.loads(body).get("ok"):
        raise Blocked(f"{boundary} rejected the doctor request")


def _compose_real(lab: Lab) -> None:
    if shutil.which("docker") is None:
        raise Skip("docker is unavailable")
    code, _ = lab.compose(["up", "-d", "postgres"])
    if code:
        raise Blocked("Compose PostgreSQL did not start")
    code, output = lab.compose(["ps", "-q", "postgres"])
    if code or not output.strip():
        raise Blocked("Compose PostgreSQL was not observed")
    code, _ = lab.compose(["down", "--volumes", "--remove-orphans"])
    if code:
        raise Blocked("Compose cleanup failed")
    lab.compose_started = False


def _protocol_player(lab: Lab) -> None:
    if os.environ.get("LKJMC_ACCEPT_MINECRAFT_EULA") != "1":
        raise Skip("LKJMC_ACCEPT_MINECRAFT_EULA=1 is required")
    if shutil.which("docker") is None or shutil.which("java") is None:
        raise Skip("docker and java are required")
    env = {"LKJMC_PLAYABLE_SMOKE": "1", "LKJMC_COMPOSE_PROJECT_NAME": lab.compose_project, "LKJMC_PLAYABLE_JAVA_PORT": str(lab.port())}
    code, _ = lab.run("protocol-player", [str(ROOT / "scripts/check-playable-smoke.sh")], 1800, env)
    if code:
        raise Blocked("protocol player smoke failed")


def secret_redaction(lab: Lab) -> None:
    token, password, basic, bearer = (uuid.uuid4().hex for _ in range(4))
    uri_secret, query_secret = "space " + uuid.uuid4().hex, "query " + uuid.uuid4().hex
    lab.secrets.update({token, password, basic, bearer})
    values = (token, password, basic, bearer, uri_secret, query_secret)
    artifact = lab.record("redaction.txt", "\n".join([
        f'{{"nested":{{"token":"{token}","password":"{password}"}}}}',
        f"Authorization: Basic {basic}", f"Authorization: Bearer {bearer}",
        f"POSTGRES_PASSWORD={password}",
        f"https://lab:{uri_secret} @lab.invalid/?token={query_secret}&safe=1",
        f"custom+lab://lab:{uri_secret} @lab.invalid/?authorization={query_secret}#done",
        f"postgres://lab:{uri_secret} @127.0.0.1/lkjmc_lab_test?api_key={query_secret}",
    ]))
    if any(value in artifact.read_text(encoding="utf-8") for value in values):
        raise Blocked("secret was retained in an artifact")


PROBES = {"postgres-real": postgres_real, "daemon-http-real": daemon_http_real, "process-real": process_real, "java-client-real": java_client_real, "isolation-cleanup": isolation_cleanup, "secret-redaction": secret_redaction}


def run(name: str) -> str:
    lab, result = Lab(name), "PASS"
    try:
        PROBES[name](lab)
    except Skip as error:
        result = "SKIP"
        lab.record("result.txt", str(error))
    except Exception as error:
        result = "BLOCKED"
        lab.record("failure.txt", f"{type(error).__name__}: {error}")
    finally:
        lab.close()
    try:
        if result == "PASS" and name == "isolation-cleanup":
            cleanup_proved(lab)
    except Exception as error:
        result = "BLOCKED"
        lab.record("failure.txt", f"{type(error).__name__}: {error}")
    print(f"{result} {name} artifacts={lab.artifacts}")
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--probe", choices=["all", *PROBES], default="all")
    name = parser.parse_args().probe
    results = [run(item) for item in (PROBES if name == "all" else [name])]
    return int("BLOCKED" in results)


if __name__ == "__main__":
    raise SystemExit(main())
