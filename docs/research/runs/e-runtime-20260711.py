#!/usr/bin/env python3
"""Research-only E-RUNTIME runner; it does not invoke a daemon command."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
from e_runtime_20260711_local import local_run
from e_runtime_20260711_postgres import postgres_run
from e_runtime_20260711_recovery_run import recovery_run

ROOT = Path(__file__).resolve().parents[3]


def clean(text):
    text = re.sub(r"(?i)(password|token|secret)=\S+", r"\1=[REDACTED]", text)
    text = re.sub(r"(?i)bearer\s+\S+", "Bearer [REDACTED]", text)
    return text[:8192]


def capture(raw, label, args, timeout=90):
    try:
        result = subprocess.run(args, cwd=ROOT, text=True, capture_output=True, timeout=timeout, check=False)
        out = clean(result.stdout + result.stderr)
        code = result.returncode
    except (OSError, subprocess.TimeoutExpired) as error:
        out, code = clean(str(error)), 127
    (raw / f"{label}.txt").write_text(out, encoding="utf-8")
    return {"code": code, "out": out, "command": args}


def index(raw):
    files = {}
    for path in sorted(raw.iterdir()):
        if path.is_file() and path.name != "index.json":
            files[path.name] = hashlib.sha256(path.read_bytes()).hexdigest()
    (raw / "index.json").write_text(json.dumps({"files": files}, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def run(compose):
    raw = Path(tempfile.mkdtemp(prefix="lkjmc-e-runtime-"))
    command = lambda label, args, timeout=90: capture(raw, label, args, timeout)
    toolchain = {"python": command("python-version", [sys.executable, "--version"])["out"],
                 "rustc": command("rustc-version", ["rustc", "--version"])["out"],
                 "cargo": command("cargo-version", ["cargo", "--version"])["out"]}
    safety = {
        "local-process-safety": command("local-process-safety", ["cargo", "test", "-p", "lkjmc-daemon", "runtime::local::tests", "--", "--nocapture"], 300)["code"],
        "kubernetes-planner": command("kubernetes-planner", ["cargo", "test", "-p", "lkjmc-core", "kubernetes_tests", "--", "--nocapture"], 300)["code"],
    }
    result = {"base": "d20e5e532db9d3a5577f567dd6a5a24fdc51eea1", "seed": 20260711,
              "environment": {"platform": platform.platform(), "toolchain": toolchain},
              "safety": {name: "PASS" if code == 0 else "FAIL" for name, code in safety.items()},
              "candidates": local_run(raw), "recovery": recovery_run(raw)}
    result["advisory"] = postgres_run(raw, ROOT, command) if compose else {"state": "NOT_ATTEMPTED"}
    after = {
        "local-process-safety": command("local-process-safety-post", ["cargo", "test", "-p", "lkjmc-daemon", "runtime::local::tests", "--", "--nocapture"], 300)["code"],
        "kubernetes-planner": command("kubernetes-planner-post", ["cargo", "test", "-p", "lkjmc-core", "kubernetes_tests", "--", "--nocapture"], 300)["code"],
    }
    result["safety_after"] = {name: "PASS" if code == 0 else "FAIL" for name, code in after.items()}
    failed = any(state == "FAIL" for state in result["safety"].values())
    failed = failed or any(state == "FAIL" for state in result["safety_after"].values())
    failed = failed or any(row["state"] == "FAIL" for row in result["candidates"].values())
    failed = failed or result["recovery"]["state"] != "PASS"
    failed = failed or result["advisory"]["state"] == "FAIL"
    result["overall"] = "FAIL" if failed else "PASS" if result["advisory"]["state"] == "PASS" else "PASS_WITH_BLOCKED_EXTERNAL"
    (raw / "summary.json").write_text(json.dumps(result, sort_keys=True, indent=2) + "\n", encoding="utf-8")
    index(raw)
    print(f"E-RUNTIME overall={result['overall']} raw={raw}")
    return 1 if failed else 0


def owned(raw):
    parent = Path(tempfile.gettempdir()).resolve()
    return raw.exists() and not raw.is_symlink() and raw.parent.resolve() == parent and raw.name.startswith("lkjmc-e-runtime-")


def replay(raw):
    if not owned(raw) or not (raw / "index.json").is_file():
        print("E-RUNTIME replay=BLOCKED index=missing-or-unsafe")
        return 2
    recorded = json.loads((raw / "index.json").read_text(encoding="utf-8"))["files"]
    actual = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in raw.iterdir() if path.is_file() and path.name != "index.json"}
    state = "PASS" if actual == recorded else "BLOCKED"
    print(f"E-RUNTIME replay={state} raw={raw}")
    return 0 if state == "PASS" else 2


def cleanup(raw):
    if not owned(raw):
        print("E-RUNTIME cleanup=BLOCKED unsafe-root")
        return 2
    shutil.rmtree(raw)
    print("E-RUNTIME cleanup=PASS")
    return 0


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("run", "replay", "cleanup"))
    parser.add_argument("--compose", action="store_true")
    parser.add_argument("--raw-dir", type=Path)
    args = parser.parse_args()
    if args.action == "run":
        return run(args.compose)
    if args.raw_dir is None:
        parser.error("--raw-dir is required for replay and cleanup")
    return replay(args.raw_dir) if args.action == "replay" else cleanup(args.raw_dir)


if __name__ == "__main__":
    sys.exit(main())
