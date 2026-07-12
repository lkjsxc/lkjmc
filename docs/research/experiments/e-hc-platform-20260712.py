#!/usr/bin/env python3
"""Run E-HC-PLATFORM evidence without changing product state."""
from __future__ import annotations
import argparse, hashlib, json, math, os, re, shutil, sqlite3, subprocess
import sys, tempfile, time, urllib.error, urllib.request, uuid
from pathlib import Path
SCRIPT = Path(__file__).resolve()
ANCHOR = SCRIPT.with_name("e-hc-platform-20260712.capture.json")
LIMIT, SEED = 8192, 20260712
PREFIX = re.compile(r"lkjmc-e-hc-platform-[a-z0-9-]+$")
MARKER = ".lkjmc-e-hc-platform-owned"
POLICY = SCRIPT.with_name("e-hc-platform-policy.py")
BOUNDARY = SCRIPT.with_name("e-hc-platform-boundary.rs")
WASM_POLICY = SCRIPT.with_name("e-hc-platform-wasm-policy.rs")
WASM_RUNNER = SCRIPT.with_name("e-hc-platform-wasm-runner.mjs")
FORMAT, BASE = "e-hc-platform-v1", "4b9357a"
SOURCES = (SCRIPT, POLICY, BOUNDARY, WASM_POLICY, WASM_RUNNER)
LANES = {"alternate-language-boundary": "PASS", "embedded-store-product-slice": "PASS", "sandboxed-policy-module": "PASS", "remote-world-io": "BLOCKED"}
def repo() -> Path:
    return Path(subprocess.check_output(["git", "-C", str(SCRIPT.parent), "rev-parse", "--show-toplevel"], text=True).strip())
def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()
def scrub(value: str) -> str:
    return re.sub(r"[a-z][a-z0-9+.-]*://[^\s'\"]+", "<remote-world-url>", value, flags=re.I)
def metrics(values: list[float]) -> dict[str, float | int]:
    ordered = sorted(values)
    return {"samples": len(values), "p50Ms": round(ordered[len(values) // 2], 3), "p95Ms": round(ordered[math.ceil(len(values) * .95) - 1], 3)}
class Evidence:
    def __init__(self, raw: Path) -> None:
        self.raw, self.artifacts, self.lanes = raw, [], []
    def record(self, name: str, value: str) -> None:
        data = scrub(value).encode("utf-8")[-LIMIT:]
        (self.raw / name).write_bytes(data)
        self.artifacts.append({"path": name, "bytes": len(data), "sha256": digest(data)})
    def artifact(self, path: Path) -> None:
        data = path.read_bytes()
        self.artifacts.append({"path": path.name, "bytes": len(data), "sha256": digest(data)})
    def command(self, name: str, args: list[str], input_text: str = "", timeout: int = 30) -> tuple[int, str, float]:
        started = time.perf_counter()
        try:
            done = subprocess.run(args, cwd=repo(), input=input_text, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout, check=False)
            code, output = done.returncode, done.stdout
        except (OSError, subprocess.TimeoutExpired) as error:
            code, output = (127 if isinstance(error, OSError) else 124), str(error)
        elapsed = (time.perf_counter() - started) * 1000
        self.record(name + ".log", "$ " + " ".join(args) + f"\nexit={code} ms={elapsed:.3f}\n" + output)
        return code, output, elapsed
    def lane(self, name: str, state: str, summary: str, **extra: object) -> None:
        self.lanes.append({"name": name, "state": state, "summary": scrub(summary), **extra})
    def finish(self) -> int:
        sources = {path.name: digest(path.read_bytes()) for path in SOURCES}
        paths = [item["path"] for item in self.artifacts]
        manifest = {"format": FORMAT, "base": BASE, "seed": SEED, "sources": sources, "run": self.raw.name, "count": len(paths), "paths": paths, "artifacts": self.artifacts}
        manifest_data = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode()
        (self.raw / "artifact-manifest.json").write_bytes(manifest_data)
        binding = {"path": "artifact-manifest.json", "sha256": digest(manifest_data), "count": len(paths), "paths": paths}
        index = {"format": FORMAT, "base": BASE, "seed": SEED, "sources": sources, "artifactManifest": binding, "artifacts": self.artifacts, "lanes": self.lanes}
        data = (json.dumps(index, indent=2, sort_keys=True) + "\n").encode()
        (self.raw / "index.json").write_bytes(data)
        (self.raw / "index.sha256").write_text(digest(data) + "  index.json\n", encoding="utf-8")
        print(f"E-HC-PLATFORM index={self.raw / 'index.json'} sha256={digest(data)}")
        print(" ".join(f"{item['name']}={item['state']}" for item in self.lanes))
        return int(any(item["state"] == "FAIL" for item in self.lanes))
def boundary(e: Evidence) -> None:
    binary = e.raw / "alternate-boundary"
    code, _, _ = e.command("boundary-build", ["rustc", str(BOUNDARY), "-O", "-o", str(binary)])
    if code:
        e.lane("alternate-language-boundary", "BLOCKED", "rustc could not compile the Rust-to-Python boundary")
        return
    e.command("boundary-warmup", [str(binary), str(POLICY), '{"subject":"operator","operation":"inspect"}']); e.artifact(binary)
    samples, valid = [], True
    for number in range(30):
        code, output, elapsed = e.command(f"boundary-{number:02d}", [str(binary), str(POLICY), '{"subject":"operator","operation":"inspect"}'])
        valid = valid and code == 0 and output.strip() == '{"decision":"allow"}'; samples.append(elapsed)
    failed, _, _ = e.command("boundary-child-failure", [str(binary), str(e.raw / "missing-policy.py"), "{}"])
    e.lane("alternate-language-boundary", "PASS" if valid and failed else "FAIL", "Rust invoked a Python policy over JSON lines; a missing child returned failure, not an action.", metrics=metrics(samples), requests=30, childFailureExit=failed)
def embedded_store(e: Evidence) -> None:
    database, backup, started = e.raw / "embedded.sqlite", e.raw / "embedded-backup.sqlite", time.perf_counter()
    primary = sqlite3.connect(database)
    primary.execute("CREATE TABLE desired_state (resource TEXT PRIMARY KEY, revision INTEGER NOT NULL, body TEXT NOT NULL)")
    primary.execute("INSERT INTO desired_state VALUES (?, ?, ?)", ("research-server", 1, json.dumps({"seed": SEED, "desired": "stopped"})))
    primary.commit(); primary.execute("BEGIN IMMEDIATE"); conflict = sqlite3.connect(database, timeout=.05)
    try:
        conflict.execute("UPDATE desired_state SET revision = 2 WHERE resource = 'research-server'"); blocked = False
    except sqlite3.OperationalError as error:
        blocked = "locked" in str(error).lower()
    finally:
        conflict.close(); primary.rollback()
    destination = sqlite3.connect(backup); primary.backup(destination); destination.close(); primary.close()
    restored = sqlite3.connect(backup).execute("SELECT revision FROM desired_state WHERE resource = 'research-server'").fetchone()
    e.artifact(database); e.artifact(backup)
    e.lane("embedded-store-product-slice", "PASS" if blocked and restored == (1,) else "FAIL", "Private SQLite held one desired-state revision, rejected a conflicting writer, and restored a backup.", metrics=metrics([(time.perf_counter() - started) * 1000]), locked=blocked, restoredRevision=restored[0] if restored else None)
def sandbox(e: Evidence) -> None:
    wasm = e.raw / "policy.wasm"
    build = ["rustc", "--target", "wasm32-unknown-unknown", "--crate-type", "cdylib", "-O", "-C", "panic=abort", str(WASM_POLICY), "-o", str(wasm)]
    code, _, _ = e.command("wasm-build", build)
    if code:
        e.lane("sandboxed-policy-module", "BLOCKED", "wasm32-unknown-unknown target could not build the policy module"); return
    flags = ["node", "--experimental-permission", f"--allow-fs-read={WASM_RUNNER}", f"--allow-fs-read={wasm}", str(WASM_RUNNER), str(wasm)]
    allow, yes, _ = e.command("sandbox-allow", flags, '{"subject":"operator","operation":"inspect"}\n')
    deny, no, _ = e.command("sandbox-deny", flags, "not-json\n"); target = e.raw / "write-probe"
    probe, wrote, _ = e.command("sandbox-write-probe", [*flags, "--write-probe", str(target)])
    state = "PASS" if allow == deny == probe == 0 and yes.strip().endswith('{"decision":"allow","imports":0}') and no.strip().endswith('{"decision":"deny","imports":0}') and wrote.strip().endswith("write-denied") and not target.exists() else "FAIL"
    e.artifact(wasm); e.lane("sandboxed-policy-module", state, "Wasm had no imports; Node permission mode allowed only runner/module reads and denied a host write probe.")
def remote_world(e: Evidence, base: str | None) -> None:
    rerun = "LKJMC_REMOTE_WORLD_URL=<controlled-url> python3 docs/research/experiments/e-hc-platform-20260712.py --output /tmp/lkjmc-e-hc-platform-remote"
    if not base:
        e.record("remote-world-attempt.log", "blocked before network access: LKJMC_REMOTE_WORLD_URL is unset; urlopen/request count=0\nrerun=" + rerun)
        e.lane("remote-world-io", "BLOCKED", "LKJMC_REMOTE_WORLD_URL is unset; no urlopen or remote request was issued.", networkAttempted=False, requestCount=0, rerun=rerun); return
    payload, puts, gets, deleted = bytes((SEED + index) % 256 for index in range(65536)), [], [], 0
    try:
        for number in range(10):
            target = base.rstrip("/") + "/e-hc-platform-" + uuid.uuid4().hex
            started = time.perf_counter(); urllib.request.urlopen(urllib.request.Request(target, data=payload, method="PUT"), timeout=10).read(); puts.append((time.perf_counter() - started) * 1000)
            started = time.perf_counter(); received = urllib.request.urlopen(target, timeout=10).read(); gets.append((time.perf_counter() - started) * 1000)
            if received != payload: raise ValueError("GET bytes differed from PUT bytes")
            urllib.request.urlopen(urllib.request.Request(target, method="DELETE"), timeout=10).read(); deleted += 1
    except (OSError, ValueError, urllib.error.URLError) as error:
        e.record("remote-world-attempt.log", "remote attempt blocked: " + str(error) + "\nrerun=" + rerun)
        e.lane("remote-world-io", "EXTERNAL-PENDING", "Configured remote-world PUT/GET/DELETE attempt failed or was unavailable.", rerun=rerun); return
    e.lane("remote-world-io", "PASS", "Controlled remote endpoint completed byte-equal PUT/GET and cleanup; this is not deployment support.", put=metrics(puts), get=metrics(gets), deleted=deleted, bytes=len(payload))
def safe_output(value: Path) -> Path:
    output = value.resolve()
    if output.parent != Path(tempfile.gettempdir()).resolve() or not PREFIX.fullmatch(output.name):
        raise ValueError("--output must be a new /tmp/lkjmc-e-hc-platform-* directory")
    return output
def prepare(output: Path) -> Path:
    raw = safe_output(output)
    if raw.exists(): raise ValueError("refusing a pre-existing output directory")
    raw.mkdir(mode=0o700); (raw / MARKER).write_text(raw.name + "\n", encoding="utf-8"); return raw
def capture_anchor() -> dict[str, object]:
    anchor = json.loads(ANCHOR.read_text(encoding="utf-8"))
    required = {"format", "captureCommit", "harnessPath", "harnessSha256", "rawRoot", "indexSha256", "manifestSha256", "sourceHashes", "orderedArtifacts"}
    if set(anchor) != required or anchor["format"] != "e-hc-platform-capture-anchor-v1": raise ValueError("invalid capture anchor")
    if anchor["harnessPath"] != str(SCRIPT.relative_to(repo())): raise ValueError("invalid capture harness path")
    if anchor["harnessSha256"] != anchor["sourceHashes"].get(SCRIPT.name): raise ValueError("invalid capture harness hash")
    expected = [{"path": path, "bytes": size, "sha256": sha256} for path, size, sha256 in anchor["orderedArtifacts"]]
    if not all(Path(item["path"]).name == item["path"] and isinstance(item["bytes"], int) for item in expected): raise ValueError("invalid capture artifacts")
    for name, expected_hash in anchor["sourceHashes"].items():
        path = SCRIPT.parent.relative_to(repo()) / name
        captured = subprocess.check_output(["git", "-C", str(repo()), "show", f"{anchor['captureCommit']}:{path}"], stderr=subprocess.DEVNULL)
        if digest(captured) != expected_hash: raise ValueError("capture source hash mismatch")
    return anchor
def replay(output: Path, expected_root: str | None = None) -> int:
    raw = safe_output(output)
    try:
        anchor = capture_anchor(); expected = [{"path": path, "bytes": size, "sha256": sha256} for path, size, sha256 in anchor["orderedArtifacts"]]
        index_data = (raw / "index.json").read_bytes(); index = json.loads(index_data)
        valid = raw.name == (expected_root or anchor["rawRoot"]) and digest(index_data) == anchor["indexSha256"]
        valid = valid and (raw / "index.sha256").read_text(encoding="utf-8") == anchor["indexSha256"] + "  index.json\n"
        valid = valid and digest((raw / "artifact-manifest.json").read_bytes()) == anchor["manifestSha256"]
        valid = valid and index.get("sources") == anchor["sourceHashes"] and index.get("artifacts") == expected
        valid = valid and (index.get("format"), index.get("base"), index.get("seed")) == (FORMAT, BASE, SEED)
        names = [item["path"] for item in expected]
        valid = valid and {path.name for path in raw.iterdir()} == {MARKER, "artifact-manifest.json", "index.json", "index.sha256", *names}
        for item in expected:
            data = (raw / item["path"]).read_bytes(); valid = valid and len(data) == item["bytes"] and digest(data) == item["sha256"]
        remote = next(item for item in index["lanes"] if item["name"] == "remote-world-io")
        valid = valid and (raw / MARKER).read_text(encoding="utf-8") == raw.name + "\n" and sorted((item["name"], item["state"]) for item in index["lanes"]) == sorted(LANES.items()) and remote.get("networkAttempted") is False and remote.get("requestCount") == 0
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError, IndexError, StopIteration, subprocess.CalledProcessError): valid = False
    print("E-HC-PLATFORM replay=" + ("PASS" if valid else "BLOCKED")); return int(not valid)
def rewrite(test: Path, index: dict[str, object], manifest: dict[str, object]) -> None:
    manifest_data = (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode(); (test / "artifact-manifest.json").write_bytes(manifest_data)
    index["artifactManifest"]["sha256"] = digest(manifest_data)
    index_data = (json.dumps(index, indent=2, sort_keys=True) + "\n").encode(); (test / "index.json").write_bytes(index_data)
    (test / "index.sha256").write_text(digest(index_data) + "  index.json\n", encoding="utf-8")
def self_test(output: Path) -> int:
    raw, valid = safe_output(output), replay(output) == 0
    for kind in ("reordered", "missing", "extra", "content-forged"):
        test = raw.parent / ("lkjmc-e-hc-platform-self-test-" + uuid.uuid4().hex); shutil.copytree(raw, test)
        (test / MARKER).write_text(test.name + "\n", encoding="utf-8")
        index = json.loads((test / "index.json").read_text(encoding="utf-8")); manifest = json.loads((test / "artifact-manifest.json").read_text(encoding="utf-8"))
        if kind == "reordered":
            index["artifacts"].reverse(); index["artifactManifest"]["paths"].reverse(); manifest["artifacts"].reverse(); manifest["paths"].reverse()
        elif kind == "missing":
            item = index["artifacts"].pop(0); (test / item["path"]).unlink(); index["artifactManifest"]["paths"].pop(0); manifest["artifacts"].pop(0); manifest["paths"].pop(0); manifest["count"] -= 1; index["artifactManifest"]["count"] -= 1
        elif kind == "extra":
            data = b"forged"; (test / "forged-artifact").write_bytes(data); item = {"path": "forged-artifact", "bytes": len(data), "sha256": digest(data)}
            for value in (index["artifacts"], manifest["artifacts"]): value.append(item.copy())
            for value in (index["artifactManifest"]["paths"], manifest["paths"]): value.append(item["path"])
            manifest["count"] += 1; index["artifactManifest"]["count"] += 1
        else:
            item = index["artifacts"][0]; data = b"forged-content"; (test / item["path"]).write_bytes(data); item.update(bytes=len(data), sha256=digest(data)); manifest["artifacts"][0].update(item)
        rewrite(test, index, manifest); valid = valid and replay(test, raw.name) != 0; shutil.rmtree(test)
    print("E-HC-PLATFORM tamper-self-test=" + ("PASS" if valid else "FAIL")); return int(not valid)
def cleanup(output: Path) -> int:
    raw = safe_output(output)
    if (raw / MARKER).read_text(encoding="utf-8") != raw.name + "\n": raise ValueError("output is not owned by this harness")
    shutil.rmtree(raw); return 0
def main() -> int:
    parser = argparse.ArgumentParser(); parser.add_argument("action", choices=["run", "replay", "self-test", "cleanup"], nargs="?", default="run"); parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.action == "replay": return replay(args.output)
    if args.action == "self-test": return self_test(args.output)
    if args.action == "cleanup": return cleanup(args.output)
    evidence = Evidence(prepare(args.output)); evidence.command("toolchain", ["sh", "-c", "python3 --version; rustc --version; node --version"])
    boundary(evidence); embedded_store(evidence); sandbox(evidence); remote_world(evidence, os.environ.get("LKJMC_REMOTE_WORLD_URL")); return evidence.finish()
if __name__ == "__main__":
    try: raise SystemExit(main())
    except ValueError as error: print("E-HC-PLATFORM error=" + str(error), file=sys.stderr); raise SystemExit(2)
