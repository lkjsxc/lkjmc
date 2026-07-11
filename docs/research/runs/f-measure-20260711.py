#!/usr/bin/env python3
"""Portable, bounded F-MEASURE rerun and artifact replay."""
from __future__ import annotations
import argparse, hashlib, json, math, os, re, secrets, shlex, shutil, socket, subprocess, sys, tempfile, time, uuid
from pathlib import Path
LIMIT = 8192
TEMP_PARENT = Path(tempfile.gettempdir()).resolve()
RAW_RE = re.compile(r"lkjmc-f-measure-[0-9a-f]{32}$")
MARKER = ".lkjmc-f-measure-owned"
BEARER = re.compile(r"(?i)(\bbearer\s+)\S+")
SECRET = re.compile(r"(?i)((?:password|token|secret|api[_-]?key)\s*[:=]\s*)\S+")
SCRIPT = Path(__file__).resolve()
def root() -> Path:
    return Path(subprocess.check_output(["git", "-C", str(SCRIPT.parent), "rev-parse", "--show-toplevel"], text=True).strip()).resolve()
def scrub(value: str, token: str = "") -> str:
    return SECRET.sub(r"\1<redacted>", BEARER.sub(r"\1<redacted>", value.replace(token, "<redacted>") if token else value))
def capped(value: str, token: str = "") -> bytes:
    data = scrub(value, token).encode("utf-8"); return data if len(data) <= LIMIT else data[-LIMIT:].decode("utf-8", "ignore").encode("utf-8")
def record(raw: Path, name: str, value: str, artifacts: list[dict], token: str = "") -> None:
    data = capped(value, token); (raw / name).write_bytes(data); artifacts.append({"path": name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})
def safe_raw(value: Path) -> Path:
    supplied = value if value.is_absolute() else Path.cwd() / value
    current = Path(supplied.anchor)
    for part in supplied.parts[1:]:
        if part == "..": raise ValueError("--raw-dir must not contain path traversal")
        current /= part
        if current.is_symlink(): raise ValueError("--raw-dir must not contain symlinks")
    raw = supplied.resolve()
    if raw.parent != TEMP_PARENT or not RAW_RE.fullmatch(raw.name): raise ValueError("--raw-dir must be a unique lkjmc-f-measure-* directory below the temp parent")
    return raw
def owned(raw: Path) -> bool:
    try: return raw.is_dir() and not raw.is_symlink() and (raw / MARKER).read_text(encoding="utf-8") == raw.name + "\n"
    except OSError: return False
def prepare_raw(value: Path) -> Path:
    raw = safe_raw(value)
    if raw.exists(): raise ValueError("refusing a pre-existing raw directory; use explicit cleanup after replay")
    raw.mkdir(mode=0o700); (raw / MARKER).write_text(raw.name + "\n", encoding="utf-8"); return raw
def cleanup(value: Path) -> None:
    raw = safe_raw(value)
    if not owned(raw): raise ValueError("refusing to clean a raw directory not created by this harness")
    shutil.rmtree(raw)
def command(root_path: Path, action: str, raw: Path | None = None, compose: bool = False) -> str:
    args = ["python3", str(root_path / "docs/research/runs/f-measure-20260711.py"), action]
    if raw: args += ["--raw-dir", str(raw)]
    if compose: args.append("--compose")
    return shlex.join(args)
def invoke(command: list[str], cwd: Path, timeout: int, env: dict[str, str] | None = None) -> tuple[int, str, float]:
    started = time.monotonic()
    try:
        done = subprocess.run(command, cwd=cwd, env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout, check=False)
        return done.returncode, done.stdout, time.monotonic() - started
    except subprocess.TimeoutExpired as error:
        output = error.stdout.decode() if isinstance(error.stdout, bytes) else error.stdout or ""
        return 124, output + "\ncommand timed out", time.monotonic() - started
def lane(name: str, status: str, reason: str, prerequisite: str, rerun: str, **extra: object) -> dict:
    return {"name": name, "status": status, "reason": reason, "prerequisite": prerequisite, "rerun": rerun, **extra}
def free_port() -> int:
    with socket.socket() as listener: listener.bind(("127.0.0.1", 0)); return int(listener.getsockname()[1])
def post(port: int, token: str) -> tuple[int, str]:
    body = json.dumps({"requestId": str(uuid.uuid4()), "actor": {"kind": "cli", "name": "f-measure"}, "command": "doctor", "body": {}})
    request = ("POST /command HTTP/1.1\r\nHost: lkjmc-lab\r\nContent-Type: application/json\r\n" f"Authorization: Bearer {token}\r\nContent-Length: {len(body)}\r\nConnection: close\r\n\r\n{body}").encode()
    with socket.create_connection(("127.0.0.1", port), timeout=2) as connection:
        connection.sendall(request); response = b""
        while b"\r\n\r\n" not in response:
            chunk = connection.recv(4096)
            if not chunk: raise OSError("daemon closed before response headers")
            response += chunk
        head, payload = response.split(b"\r\n\r\n", 1)
        size = next(int(line.split(b":", 1)[1]) for line in head.split(b"\r\n")[1:] if line.lower().startswith(b"content-length:"))
        while len(payload) < size: payload += connection.recv(4096)
        return int(head.split(b" ")[1]), payload[:size].decode("utf-8", "replace")
def doctor(binary: Path, root_path: Path, raw: Path, artifacts: list[dict], rerun: str) -> dict:
    token, port, socket_path = secrets.token_hex(16), free_port(), raw / "daemon.sock"
    args = [str(binary), "--socket", str(socket_path), "--http", f"127.0.0.1:{port}", "--http-token", token, "--config-root", str(raw / "config"), "--log-root", str(raw / "logs"), "--jar-root", str(raw / "jars"), "--data-root", str(raw / "data")]
    try: process = subprocess.Popen(args, cwd=root_path, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    except OSError as error:
        record(raw, "daemon.log", str(error), artifacts, token); return lane("direct-tcp-doctor", "BLOCKED", scrub(str(error), token), "a runnable isolated daemon build", rerun)
    try:
        for _ in range(100):
            if process.poll() is not None: raise RuntimeError("daemon exited before ready")
            try:
                status, payload = post(port, token)
                if status == 200 and json.loads(payload).get("ok"): break
            except (OSError, ValueError, json.JSONDecodeError): time.sleep(0.05)
        else: raise RuntimeError("daemon TCP listener was not ready")
        samples = []
        for _ in range(30):
            started = time.perf_counter(); status, payload = post(port, token)
            if status != 200 or not json.loads(payload).get("ok"): raise RuntimeError("direct TCP doctor was rejected")
            samples.append((time.perf_counter() - started) * 1000)
        ordered = sorted(samples); metrics = {"requests": len(samples), "p50Ms": ordered[len(ordered) // 2], "p95Ms": ordered[math.ceil(len(ordered) * .95) - 1], "transport": "direct TCP; not JVM plugin traffic"}
        record(raw, "direct-tcp-doctor.json", json.dumps(metrics, sort_keys=True), artifacts, token)
        return lane("direct-tcp-doctor", "PASS", "30 serial direct TCP doctor requests observed.", "isolated cold daemon build", rerun, metrics=metrics)
    except Exception as error: return lane("direct-tcp-doctor", "BLOCKED", scrub(str(error), token), "a runnable isolated daemon build", rerun)
    finally:
        process.terminate()
        try: output, _ = process.communicate(timeout=10)
        except subprocess.TimeoutExpired: process.kill(); output, _ = process.communicate()
        record(raw, "daemon.log", output or "", artifacts, token); socket_path.unlink(missing_ok=True)
def compose(root_path: Path, raw: Path, artifacts: list[dict], enabled: bool, rerun: str) -> dict:
    prerequisite = "Docker plus a free disposable bridge subnet"
    if not enabled: return lane("docker-disposable-postgres", "SKIP", "not requested by --compose", prerequisite, rerun)
    if shutil.which("docker") is None: return lane("docker-disposable-postgres", "BLOCKED", "docker is unavailable", prerequisite, rerun)
    base = ["docker", "compose", "--project-name", "lkjmc_f_measure_" + secrets.token_hex(5), "-f", str(root_path / "docker-compose.yml")]
    code, output, elapsed = invoke([*base, "up", "-d", "postgres"], root_path, 240); record(raw, "compose-up.log", output, artifacts)
    down_code, down, _ = invoke([*base, "down", "--volumes", "--remove-orphans"], root_path, 120); record(raw, "compose-down.log", down, artifacts)
    reason = "Docker reports all predefined address pools are fully subnetted" if "all predefined address pools" in output else "Compose start or cleanup failed"
    return lane("docker-disposable-postgres", "BLOCKED" if code or down_code else "PASS", reason if code or down_code else "unique Compose PostgreSQL started and cleaned up.", prerequisite, rerun, seconds=elapsed)
def repeat(args: argparse.Namespace) -> int:
    root_path = root(); requested = args.raw_dir or Path(os.environ.get("LKJMC_F_MEASURE_RAW_DIR", TEMP_PARENT / ("lkjmc-f-measure-" + uuid.uuid4().hex)))
    raw, artifacts = prepare_raw(requested), []; rerun, compose_rerun = command(root_path, "repeat"), command(root_path, "repeat", compose=True)
    with tempfile.TemporaryDirectory(prefix="lkjmc-f-measure-cold-", dir=TEMP_PARENT) as cold:
        code, output, elapsed = invoke(["cargo", "build", "--locked", "-p", "lkjmc-cli", "-p", "lkjmc-daemon"], root_path, 1800, os.environ | {"CARGO_TARGET_DIR": cold})
        record(raw, "cold-build.log", output, artifacts); lanes = [lane("cold-build", "PASS" if not code else "BLOCKED", "isolated CARGO_TARGET_DIR was empty before cargo build.", "Cargo with locked dependencies", rerun, seconds=elapsed)]
        lanes.append(doctor(Path(cold) / "debug/lkjmc-daemon", root_path, raw, artifacts, rerun) if not code else lane("direct-tcp-doctor", "SKIP", "cold build did not produce a daemon.", "a successful isolated cold build", rerun))
    lanes += [compose(root_path, raw, artifacts, args.compose, compose_rerun), lane("postgres-write-latency", "SKIP", "no approved database metric scenario was run.", "confirmed disposable loopback PostgreSQL and an approved scenario", compose_rerun), lane("menu-render-timing", "SKIP", "existing JVM UI behavior tests expose no per-render timing.", "an approved JVM UI timing observer", shlex.join([str(root_path / "gradlew"), "--no-daemon", "--rerun-tasks", "--no-build-cache", ":platforms:jvm:common:test", "--tests", "com.lkjmc.common.ui.kernel.UiFrameBehaviorTest"])), lane("jvm-plugin-traffic", "SKIP", "direct TCP doctor is not JVM plugin traffic.", "a guarded Velocity/Paper/Folia plugin process and test client", "LKJMC_PLAYABLE_SMOKE=1 LKJMC_ACCEPT_MINECRAFT_EULA=1 " + shlex.quote(str(root_path / "scripts/check-playable-smoke.sh")))]
    encoded = json.dumps({"format": "f-measure-v4", "base": "7252f95314e746029f6ea04cd3deaf5fc4057051", "root": str(root_path), "artifacts": artifacts, "lanes": lanes}, indent=2, sort_keys=True) + "\n"
    (raw / "index.json").write_text(encoded, encoding="utf-8"); digest = hashlib.sha256(encoded.encode()).hexdigest(); (raw / "index.sha256").write_text(digest + "  index.json\n", encoding="utf-8")
    print(f"F-MEASURE index={raw / 'index.json'} sha256={digest}"); print("replay=" + command(root_path, "replay", raw)); print(" ".join(f"{item['name']}={item['status']}" for item in lanes))
    return int(any(item["status"] == "BLOCKED" for item in lanes))
def replay(args: argparse.Namespace) -> int:
    value = args.raw_dir or (Path(os.environ["LKJMC_F_MEASURE_RAW_DIR"]) if "LKJMC_F_MEASURE_RAW_DIR" in os.environ else None)
    if value is None: print("F-MEASURE replay=BLOCKED\nindex=missing"); return 1
    try:
        raw = safe_raw(value)
        if not owned(raw): raise ValueError("raw directory is not owned by this harness")
        index_bytes, expected = (raw / "index.json").read_bytes(), (raw / "index.sha256").read_text().split()[0]; index = json.loads(index_bytes); valid = hashlib.sha256(index_bytes).hexdigest() == expected
        for artifact in index["artifacts"]:
            path = (raw / artifact["path"]).resolve(); valid = valid and path.parent == raw and path.is_file() and path.stat().st_size <= LIMIT and hashlib.sha256(path.read_bytes()).hexdigest() == artifact["sha256"]
        states = " ".join(f"{item['name']}={item['status']}" for item in index["lanes"])
    except (KeyError, OSError, ValueError, json.JSONDecodeError, IndexError, TypeError): valid, states = False, "index=invalid"
    print(f"F-MEASURE replay={'PASS' if valid else 'BLOCKED'}\n{states}"); return int(not valid)
def self_test() -> int:
    token, redacted = "unregistered-token-42", scrub("arbitrary Bearer unregistered-token-42")
    assert token not in redacted and redacted == "arbitrary Bearer <redacted>"
    data = capped("界" * (LIMIT + 1)); assert len(data) <= LIMIT and data.decode("utf-8")
    try: safe_raw(TEMP_PARENT); raise AssertionError("unsafe raw parent accepted")
    except ValueError: pass
    owned_root = TEMP_PARENT / ("lkjmc-f-measure-" + uuid.uuid4().hex)
    unsafe_parent = TEMP_PARENT / ("unowned-" + uuid.uuid4().hex)
    try:
        prepare_raw(owned_root); sentinel = owned_root / "sentinel"; sentinel.write_text("keep", encoding="utf-8")
        final_link = TEMP_PARENT / ("lkjmc-f-measure-" + uuid.uuid4().hex); final_link.symlink_to(owned_root, target_is_directory=True)
        ancestor = TEMP_PARENT / ("f-measure-link-" + uuid.uuid4().hex); ancestor.symlink_to(TEMP_PARENT, target_is_directory=True)
        unsafe = unsafe_parent / "existing"; unsafe.mkdir(parents=True); unsafe_sentinel = unsafe / "sentinel"; unsafe_sentinel.write_text("keep", encoding="utf-8")
        for rejected, keep in [(final_link, sentinel), (ancestor / owned_root.name, sentinel), (unsafe, unsafe_sentinel)]:
            code, _, _ = invoke([sys.executable, str(SCRIPT), "cleanup", "--raw-dir", str(rejected)], Path(tempfile.gettempdir()), 30); assert code and keep.read_text(encoding="utf-8") == "keep"
    finally:
        final_link.unlink(missing_ok=True); ancestor.unlink(missing_ok=True); cleanup(owned_root); shutil.rmtree(unsafe_parent, ignore_errors=True)
    with tempfile.TemporaryDirectory() as cwd:
        code, output, _ = invoke([sys.executable, str(SCRIPT), "--print-root"], Path(cwd), 20); assert code == 0 and Path(output.strip()) == root()
        _, output, _ = invoke([sys.executable, str(SCRIPT), "repeat"], Path(cwd), 1800); match = re.search(r"F-MEASURE index=(.+)/index\.json sha256=[0-9a-f]+", output); assert match, output
        raw = Path(match.group(1)); index = json.loads((raw / "index.json").read_bytes()); assert raw.is_dir() and (raw / "index.sha256").is_file() and index["artifacts"] and all((raw / item["path"]).is_file() for item in index["artifacts"])
        replay_code, replay_output, _ = invoke([sys.executable, str(SCRIPT), "replay", "--raw-dir", str(raw)], Path(cwd), 30); assert replay_code == 0 and "F-MEASURE replay=PASS" in replay_output
        with (raw / index["artifacts"][0]["path"]).open("ab") as artifact: artifact.write(b"x")
        replay_code, replay_output, _ = invoke([sys.executable, str(SCRIPT), "replay", "--raw-dir", str(raw)], Path(cwd), 30); assert replay_code == 1 and "F-MEASURE replay=BLOCKED" in replay_output
        cleanup_code, _, _ = invoke([sys.executable, str(SCRIPT), "cleanup", "--raw-dir", str(raw)], Path(cwd), 30); assert cleanup_code == 0 and not raw.exists()
    print("ok f-measure self-test"); return 0
def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("action", choices=["repeat", "replay", "cleanup"], nargs="?"); parser.add_argument("--raw-dir", type=Path); parser.add_argument("--compose", action="store_true"); parser.add_argument("--self-test", action="store_true"); parser.add_argument("--print-root", action="store_true")
    args = parser.parse_args()
    if args.print_root: print(root()); return 0
    if args.self_test: return self_test()
    if not args.action: parser.error("action is required unless --self-test is used")
    try:
        if args.action == "cleanup":
            if args.raw_dir is None: raise ValueError("cleanup requires --raw-dir")
            cleanup(args.raw_dir); return 0
        return repeat(args) if args.action == "repeat" else replay(args)
    except ValueError as error: parser.error(str(error))
if __name__ == "__main__": raise SystemExit(main())
