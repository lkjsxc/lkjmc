#!/usr/bin/env python3
"""Run the bounded, secret-safe E-OBS research comparison outside product code."""
from __future__ import annotations
import argparse, hashlib, json, math, os, re, secrets, shutil, socket, statistics, subprocess, tempfile, time, uuid
from pathlib import Path
BASE = "d20e5e532db9d3a5577f567dd6a5a24fdc51eea1"
LIMIT, PREFIX = 16384, "lkjmc-e-obs-"
SCRIPT = Path(__file__).resolve()
def repo() -> Path:
    return Path(subprocess.check_output(["git", "-C", str(SCRIPT.parent), "rev-parse", "--show-toplevel"], text=True).strip())
def safe(root: Path) -> bool:
    return root.parent == Path(tempfile.gettempdir()).resolve() and re.fullmatch(r"lkjmc-e-obs-[0-9a-f]{32}", root.name) is not None
def owned(root: Path) -> bool:
    try: return safe(root) and (root / ".owned").read_text() == root.name + "\n"
    except OSError: return False
def scrub(value: str, secret_values: set[str]) -> str:
    for value_secret in sorted(secret_values, key=len, reverse=True): value = value.replace(value_secret, "<redacted>")
    value = re.sub(r"(?i)(authorization:\s*(?:bearer|basic)\s+)\S+", r"\1<redacted>", value)
    return re.sub(r"(?i)((?:password|token|secret)\s*[:=]\s*)\S+", r"\1<redacted>", value)
def put(root: Path, name: str, value: object, secret_values: set[str], artifacts: list[dict]) -> int:
    text = value if isinstance(value, str) else json.dumps(value, sort_keys=True, indent=2)
    data = scrub(text, secret_values).encode("utf-8")[-LIMIT:]
    (root / name).write_bytes(data); artifacts.append({"path": name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
    return len(data)
def canary_hits(root: Path, canary: str) -> list[str]:
    needle = canary.encode()
    return [path.relative_to(root).as_posix() for path in sorted(root.rglob("*")) if path.is_file() and needle in path.read_bytes()]
def prove_canary_scanner(root: Path, canary: str) -> None:
    seed = root / "canary-positive-disposable.txt"; seed.write_text(canary)
    try:
        if canary_hits(root, canary) != [seed.name]: raise RuntimeError("canary positive control was not detected")
    finally: seed.unlink(missing_ok=True)
def run(args: list[str], timeout: int, env: dict[str, str] | None = None) -> tuple[int, str]:
    try:
        done = subprocess.run(args, cwd=repo(), env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout, check=False)
        return done.returncode, done.stdout
    except subprocess.TimeoutExpired as error:
        return 124, (error.stdout or "") + "\ncommand timed out"
def port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0)); return int(listener.getsockname()[1])
def http(port_value: int, body: dict, token: str, timeout: float = 5) -> tuple[int, dict]:
    encoded = json.dumps(body).encode(); auth = f"Authorization: Bearer {token}\r\n".encode() if token else b""
    request = b"POST /command HTTP/1.1\r\nHost: lkjmc-e-obs\r\nContent-Type: application/json\r\n" + auth + f"Content-Length: {len(encoded)}\r\nConnection: close\r\n\r\n".encode() + encoded
    with socket.create_connection(("127.0.0.1", port_value), timeout=timeout) as connection:
        connection.settimeout(timeout); connection.sendall(request); raw = b""
        while b"\r\n\r\n" not in raw: raw += connection.recv(4096)
        head, payload = raw.split(b"\r\n\r\n", 1); size = next(int(line.split(b":", 1)[1]) for line in head.split(b"\r\n")[1:] if line.lower().startswith(b"content-length:"))
        while len(payload) < size: payload += connection.recv(4096)
    return int(head.split()[1]), json.loads(payload[:size])

def request(port_value: int, token: str, sequence: int, command: str, body: dict, actor: str = "cli", timeout: float = 15) -> dict:
    request_id = str(uuid.uuid5(uuid.NAMESPACE_URL, f"e-obs-20260711/{sequence}/{command}")); began = time.perf_counter()
    try:
        status, response = http(port_value, {"requestId": request_id, "actor": {"kind": actor, "name": "e-obs"}, "command": command, "body": body}, token, timeout)
        code = (response.get("error") or {}).get("code", "ok" if response.get("ok") else f"http-{status}")
        response_id = response.get("requestId") if isinstance(response.get("requestId"), str) else None
        return {"clientRequestId": request_id, "daemonResponseId": response_id, "command": command, "status": status, "ok": response.get("ok", False), "code": code, "ms": round((time.perf_counter() - began) * 1000, 3), "body": response.get("body", {})}
    except (AttributeError, OSError, ValueError, json.JSONDecodeError, StopIteration):
        return {"clientRequestId": request_id, "daemonResponseId": None, "command": command, "status": 0, "ok": False, "code": "transport.timeout_or_disconnect", "ms": round((time.perf_counter() - began) * 1000, 3), "body": {}}

def event(row: dict, fault: str | None = None) -> dict:
    return {"event": "daemon.command.completed", "clientRequestId": row["clientRequestId"], "daemonResponseId": row["daemonResponseId"], "observerEventId": str(uuid.uuid4()), "observerEventProvenance": "synthetic-harness", "command": row["command"], "result": row["code"], "durationMs": row["ms"], "fault": fault}

def id_present(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip())
def correlation(rows: list[dict], events: list[dict]) -> dict:
    pairs = bool(rows) and len(rows) == len(events) and all(id_present(row.get("clientRequestId")) and id_present(row.get("daemonResponseId")) and id_present(item.get("observerEventId")) and item.get("clientRequestId") == row["clientRequestId"] and item.get("daemonResponseId") == row["daemonResponseId"] for row, item in zip(rows, events))
    provenance = pairs and all(item.get("observerEventProvenance") == "externally-emitted" for item in events)
    reason = "PASS" if provenance else "BLOCKED: mismatched or absent client/daemon/observer ID fields" if not pairs else "BLOCKED: unsupported; no independently emitted observer event provenance on this daemon HTTP boundary"
    return {"pairedRecordsValid": pairs, "observerEventProvenance": "externally-emitted" if provenance else "unavailable", "correlation-end-to-end": reason}

def metrics(rows: list[dict]) -> dict:
    series: dict[str, int] = {}
    for row in rows:
        key = f"command={row['command']},result={row.get('code', row['result'])}"; series[key] = series.get(key, 0) + 1
    return {"counter": series, "latencyBucketsMs": [1, 10, 100, 1000], "series": len(series), "labels": ["command", "result"]}

def median_p95(values: list[float]) -> dict:
    ordered = sorted(values); return {"medianMs": round(statistics.median(ordered), 3), "p95Ms": round(ordered[math.ceil(len(ordered) * .95) - 1], 3)}

def lifecycle(port_value: int, token: str, secret_file: Path, label: str, start_sequence: int, observed: list[dict] | None) -> tuple[list[float], int, list[dict]]:
    times, rows, sequence = [], [], start_sequence
    for repeat in range(5):
        instance_id, began = f"eobs-{label}-{repeat}", time.perf_counter()
        body = {"id": instance_id, "kind": "velocity", "template": "velocity-modern", "command": "sleep 30", "memoryMb": 256, "serverPort": 34000 + start_sequence + repeat, "forwardingSecretFile": str(secret_file)}
        for command, payload in [("instance.create", body), ("instance.start", {"id": instance_id}), ("status", {}), ("instance.stop", {"id": instance_id})]:
            row = request(port_value, token, sequence, command, payload); sequence += 1; rows.append(row)
            if observed is not None: observed.append(event(row))
            if not row["ok"]: raise RuntimeError(f"{label} {command} returned {row['code']}")
        times.append((time.perf_counter() - began) * 1000)
    return times, sequence, rows

def delay_fault(container: str, port_value: int, token: str, secret_file: Path, sequence: int) -> tuple[dict, int, str]:
    code, output = run(["docker", "exec", container, "psql", "-U", "lkjmc", "-d", "lkjmc", "-v", "ON_ERROR_STOP=1", "-c", "SELECT pg_sleep(1)"], 5)
    row = request(port_value, token, sequence, "instance.create", {"id": "eobs-delay", "kind": "velocity", "template": "velocity-modern", "command": "sleep 30", "memoryMb": 256, "serverPort": 34501, "forwardingSecretFile": str(secret_file)})
    return row, sequence + 1, f"database-delay-query-exit={code}\n{output}"

def partial_disconnect(port_value: int, token: str) -> None:
    body = json.dumps({"requestId": str(uuid.uuid4()), "actor": {"kind": "velocity-plugin", "name": "e-obs"}, "command": "instance.list", "body": {}}).encode()
    with socket.create_connection(("127.0.0.1", port_value), timeout=2) as connection:
        connection.sendall(b"POST /command HTTP/1.1\r\nHost: eobs\r\nAuthorization: Bearer " + token.encode() + b"\r\nContent-Length: " + str(len(body)).encode() + b"\r\n\r\n" + body[:8])

def faults(container: str, port_value: int, token: str, secret_file: Path, sequence: int, observed: list[dict]) -> tuple[list[dict], int, str]:
    print("E-OBS fault=database-delay", flush=True); results = []; row, sequence, lock = delay_fault(container, port_value, token, secret_file, sequence); observed.append(event(row, "database-delay")); results.append({"fault": "database-delay", "command": row["command"], "result": row["code"], "latencyMs": row["ms"]})
    print("E-OBS fault=timeout-continuation", flush=True); create = request(port_value, token, sequence, "instance.create", {"id": "eobs-timeout", "kind": "velocity", "template": "velocity-modern", "command": "sleep 30", "memoryMb": 256, "serverPort": 34502, "forwardingSecretFile": str(secret_file)}); sequence += 1
    row = request(port_value, token, sequence, "instance.start", {"id": "eobs-timeout"}, timeout=.05); sequence += 1; time.sleep(.7); continued = request(port_value, token, sequence, "instance.list", {}); sequence += 1; observed.extend([event(create, "timeout-continuation"), event(row, "timeout-continuation"), event(continued, "timeout-continuation")]); results.append({"fault": "timeout-continuation", "startResult": row["code"], "followup": continued["code"]}); request(port_value, token, sequence, "instance.stop", {"id": "eobs-timeout"}); sequence += 1
    print("E-OBS fault=process-failure", flush=True); create = request(port_value, token, sequence, "instance.create", {"id": "eobs-process-fail", "kind": "velocity", "template": "velocity-modern", "command": "exit 17", "memoryMb": 256, "serverPort": 34503, "forwardingSecretFile": str(secret_file)}); sequence += 1; row = request(port_value, token, sequence, "instance.start", {"id": "eobs-process-fail"}); sequence += 1; observed.extend([event(create, "process-failure"), event(row, "process-failure")]); results.append({"fault": "process-failure", "result": row["code"]})
    print("E-OBS fault=auth-denial", flush=True); row = request(port_value, "", sequence, "instance.list", {}); sequence += 1; observed.append(event(row, "auth-denial")); results.append({"fault": "auth-denial", "httpStatus": row["status"], "result": row["code"]})
    print("E-OBS fault=plugin-disconnect", flush=True); partial_disconnect(port_value, token); row = request(port_value, token, sequence, "instance.list", {}, actor="velocity-plugin"); sequence += 1; observed.append(event(row, "plugin-disconnect")); results.append({"fault": "plugin-disconnect", "partialRequest": "sent_then_closed", "result": row["code"], "status": "BLOCKED: Java daemon adapter withdrawn"})
    return results, sequence, lock

def replay(raw: Path) -> int:
    try:
        if not owned(raw): raise ValueError("unowned root")
        index = (raw / "index.json").read_bytes(); digest = (raw / "index.sha256").read_text().split()[0]; valid = hashlib.sha256(index).hexdigest() == digest
        for item in json.loads(index)["artifacts"]: valid = valid and hashlib.sha256((raw / item["path"]).read_bytes()).hexdigest() == item["sha256"]
        scan = json.loads((raw / "secret-canary-scan.json").read_text())
        files = sorted(path.relative_to(raw).as_posix() for path in raw.rglob("*") if path.is_file())
        valid = valid and scan["positiveControlDetected"] and scan["retainedFilesClean"] and scan["scannedFiles"] == files
    except (OSError, ValueError, KeyError, json.JSONDecodeError, IndexError): valid = False
    print(f"E-OBS replay={'PASS' if valid else 'BLOCKED'} (artifact integrity)"); return int(not valid)

def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("action", choices=["repeat", "replay", "cleanup"], nargs="?", default="repeat"); parser.add_argument("--raw-dir", type=Path); parser.add_argument("--self-test", action="store_true"); args = parser.parse_args()
    if args.self_test:
        row = {"clientRequestId": "client-1", "daemonResponseId": "daemon-1"}; observed = {**row, "observerEventId": "observer-1", "observerEventProvenance": "externally-emitted"}
        assert "secret" not in scrub("token=secret", {"secret"}); assert median_p95([1, 2, 3, 4, 5])["p95Ms"] == 5; assert correlation([row], [observed])["correlation-end-to-end"] == "PASS"
        assert correlation([row], [{**observed, "daemonResponseId": "wrong"}])["correlation-end-to-end"].startswith("BLOCKED"); assert correlation([row], [{key: value for key, value in observed.items() if key != "observerEventId"}])["correlation-end-to-end"].startswith("BLOCKED"); assert correlation([], [])["correlation-end-to-end"].startswith("BLOCKED")
        for field in ("clientRequestId", "daemonResponseId", "observerEventId"):
            blank_row = {**row, field: ""}; blank_event = {**observed, field: ""}
            assert correlation([blank_row], [blank_event])["correlation-end-to-end"].startswith("BLOCKED")
        with tempfile.TemporaryDirectory() as directory: prove_canary_scanner(Path(directory), "known-positive")
        print("ok e-obs self-test"); return 0
    if args.action == "replay": return replay(args.raw_dir) if args.raw_dir else 1
    if args.action == "cleanup":
        if not args.raw_dir or not owned(args.raw_dir): return 1
        shutil.rmtree(args.raw_dir); return 0
    raw = Path(tempfile.gettempdir()).resolve() / f"{PREFIX}{uuid.uuid4().hex}"; raw.mkdir(mode=0o700); (raw / ".owned").write_text(raw.name + "\n"); artifacts: list[dict] = []; secret_values: set[str] = set(); daemon = None; container = ""; result: dict = {"format": "e-obs-v1", "base": BASE, "root": str(repo()), "state": "BLOCKED"}
    try:
        if shutil.which("docker") is None: raise RuntimeError("docker is unavailable; real disposable PostgreSQL attempt cannot start")
        password, token, canary = secrets.token_hex(16), secrets.token_hex(16), secrets.token_hex(16); secret_values.update({password, token, canary, "wrong-credential"}); db_port, http_port = port(), port(); container = f"eobs{uuid.uuid4().hex[:16]}"; code, output = run(["docker", "run", "--rm", "-d", "--name", container, "-e", "POSTGRES_DB=lkjmc", "-e", "POSTGRES_USER=lkjmc", "-e", f"POSTGRES_PASSWORD={password}", "-p", f"127.0.0.1:{db_port}:5432", "postgres:16-alpine"], 120); put(raw, "postgres-start.log", output, secret_values, artifacts)
        if code: raise RuntimeError("docker run postgres failed")
        for _ in range(60):
            code, output = run(["docker", "exec", container, "pg_isready", "-U", "lkjmc", "-d", "lkjmc"], 5)
            if not code: break
            time.sleep(.25)
        if code: raise RuntimeError("PostgreSQL did not become ready")
        url = f"postgres://lkjmc:{password}@127.0.0.1:{db_port}/lkjmc"
        with tempfile.TemporaryDirectory(prefix="lkjmc-e-obs-build-") as build, tempfile.TemporaryDirectory(prefix="lkjmc-e-obs-runtime-") as runtime:
            env = os.environ | {"CARGO_TARGET_DIR": build, "LKJMC_DATABASE_URL": url}; code, output = run(["cargo", "build", "--locked", "-p", "lkjmc-cli", "-p", "lkjmc-daemon"], 1800, env); put(raw, "cold-build.log", output, secret_values, artifacts)
            if code: raise RuntimeError("isolated cargo build failed")
            cli, daemon_bin = Path(build) / "debug/lkjmc", Path(build) / "debug/lkjmc-daemon"; code, output = run([str(cli), "db", "migrate"], 120, env); put(raw, "migrate.log", output, secret_values, artifacts)
            if code: raise RuntimeError("migration failed")
            secret_file = Path(runtime) / "forwarding.secret"; secret_file.write_text(canary + "\n"); socket_path = Path(runtime) / "daemon.sock"; daemon = subprocess.Popen([str(daemon_bin), "--socket", str(socket_path), "--http", f"127.0.0.1:{http_port}", "--http-token", token, "--database-url", url, "--config-root", runtime, "--log-root", str(Path(runtime) / "logs"), "--jar-root", str(Path(runtime) / "jars"), "--data-root", str(Path(runtime) / "data")], cwd=repo(), stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT, text=True)
            for sequence in range(80):
                ready = request(http_port, token, sequence, "doctor", {})
                if ready["ok"]: break
                time.sleep(.05)
            if not ready["ok"]: raise RuntimeError("daemon HTTP listener did not become ready")
            code, output = run(["cargo", "tree", "--locked", "-p", "lkjmc-daemon", "-i", "opentelemetry"], 120); put(raw, "telemetry-library-attempt.log", output, secret_values, artifacts)
            baseline, sequence, _ = lifecycle(http_port, token, secret_file, "baseline", 100, None); variants, observed, lifecycle_rows = {"baseline": median_p95(baseline)}, [], []
            for label in ["events", "metrics", "bundle"]:
                began = time.perf_counter(); times, sequence, rows = lifecycle(http_port, token, secret_file, label, sequence, observed); lifecycle_rows.extend(rows); record = {"events": [item for item in observed if item["clientRequestId"] in {row["clientRequestId"] for row in rows}]}; size = put(raw, f"{label}-events.json", record, secret_values, artifacts)
                if label != "events": size += put(raw, f"{label}-metrics.json", metrics(record["events"]), secret_values, artifacts)
                if label == "bundle":
                    audit = request(http_port, token, sequence, "audit.tail", {"lines": 200}); sequence += 1; status = request(http_port, token, sequence, "status", {}); sequence += 1; logs = request(http_port, token, sequence, "instance.logs", {"id": "eobs-bundle-4", "lines": 10}); sequence += 1; bundle = {"audit": audit["body"], "status": status["body"], "logs": logs["body"], "artifactIdentity": f"base={BASE}", "historyHasRequestId": False}; size += put(raw, "bundle-manifest.json", bundle, secret_values, artifacts)
                summary = median_p95(times); variants[label] = {**summary, "overheadVsBaselineMs": round(summary["medianMs"] - variants["baseline"]["medianMs"], 3), "variantWallMs": round((time.perf_counter() - began) * 1000, 3), "artifactBytes": size, "eventFields": 9, "metricSeries": metrics(record["events"])["series"] if label != "events" else 0}
            fault_events: list[dict] = []; fault_rows, sequence, lock = faults(container, http_port, token, secret_file, sequence, fault_events); put(raw, "fault-lock.log", lock, secret_values, artifacts); diagnostic = {}
            for label in ["events", "metrics", "bundle"]:
                began = time.perf_counter(); view = fault_events if label == "events" else metrics(fault_events) if label == "metrics" else {"events": fault_events, "audit": request(http_port, token, sequence, "audit.tail", {"lines": 200})["body"]}; sequence += label == "bundle"; diagnostic[label] = {"lookupMs": round((time.perf_counter() - began) * 1000, 3), "faultsVisible": len(fault_events), "view": "events" if label == "events" else "fixed labels" if label == "metrics" else "events plus audit history"}
            put(raw, "faults.json", {"faults": fault_rows, "diagnostic": diagnostic}, secret_values, artifacts)
            correlation_record = {**correlation(lifecycle_rows, observed), "pairedLifecycleEvents": len(observed), "auditHistoryRequestId": False}
            put(raw, "correlation.json", correlation_record, secret_values, artifacts)
            if daemon:
                daemon.terminate()
                try: daemon.wait(timeout=10)
                except subprocess.TimeoutExpired: daemon.kill(); daemon.wait()
                put(raw, "daemon.log", "daemon stdout deliberately not retained", secret_values, artifacts); daemon = None
            if container:
                cleanup_code, output = run(["docker", "rm", "-f", container], 30); put(raw, "postgres-cleanup.log", output, secret_values, artifacts); result["cleanupExit"] = cleanup_code; container = ""
            prove_canary_scanner(raw, canary)
            clean = not canary_hits(raw, canary)
            files = {path.relative_to(raw).as_posix() for path in raw.rglob("*") if path.is_file()}
            put(raw, "secret-canary-scan.json", {"positiveControlDetected": True, "retainedFilesClean": clean, "scannedFiles": sorted(files | {"secret-canary-scan.json", "index.json", "index.sha256"})}, secret_values, artifacts)
            result.update({"state": "BLOCKED", "blocker": correlation_record["correlation-end-to-end"], "variants": variants, "faults": fault_rows, "diagnostic": diagnostic, "correlation": correlation_record, "telemetryLibraryAttemptExit": code, "artifactBytes": sum(item["bytes"] for item in artifacts)})
    except Exception as error:
        put(raw, "blocked.json", {"error": type(error).__name__, "detail": str(error)}, secret_values, artifacts); result["blocker"] = scrub(str(error), secret_values)
    finally:
        if daemon is not None:
            daemon.terminate()
            try: daemon.wait(timeout=10)
            except subprocess.TimeoutExpired: daemon.kill(); daemon.wait()
            put(raw, "daemon.log", "daemon stdout deliberately not retained", secret_values, artifacts)
        if container: code, output = run(["docker", "rm", "-f", container], 30); put(raw, "postgres-cleanup.log", output, secret_values, artifacts); result["cleanupExit"] = code
    result["artifacts"] = artifacts; encoded = json.dumps(result, sort_keys=True, indent=2).encode() + b"\n"; (raw / "index.json").write_bytes(encoded); (raw / "index.sha256").write_text(hashlib.sha256(encoded).hexdigest() + "  index.json\n")
    if "canary" in locals() and canary_hits(raw, canary): raise RuntimeError("secret canary leaked into retained evidence")
    print(f"E-OBS index={raw / 'index.json'}\nreplay=python3 {SCRIPT} replay --raw-dir {raw}"); return int(result["state"] != "PASS")

if __name__ == "__main__": raise SystemExit(main())
