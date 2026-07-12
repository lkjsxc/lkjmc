#!/usr/bin/env python3
"""Collect bounded E-PRODUCT local-model and blocked-surface evidence."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[3]
CAP = 8192
CURATED = (
    "docs/product/journeys.md",
    "docs/product/gui/docs-browser.md",
    "docs/product/player-help.md",
    "docs/product/identity-onboarding.md",
    "docs/product/commands/minecraft.md",
    "docs/product/i18n/ownership.md",
)


def redact(text):
    text = re.sub(r"(?i)(bearer|token|password)\s*[=:]\s*\S+", r"\1=<redacted>", text)
    text = re.sub(r"(https?://)[^\s/@:]+:[^\s/@]+@", r"\1<redacted>@", text)
    return text[:CAP]


def command(name, args, env=None, blocked=False, expected=(0,), marker=""):
    started = time.monotonic()
    values = os.environ.copy()
    values.update(env or {})
    run = subprocess.run(args, cwd=ROOT, env=values, text=True,
                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                         check=False)
    output = redact(run.stdout)
    ok = (run.returncode != 0 and "blocked:" in output) if blocked else run.returncode in expected
    if marker:
        ok = ok and marker in output
    return {
        "name": name,
        "command": " ".join(args),
        "exit": run.returncode,
        "elapsedSeconds": round(time.monotonic() - started, 3),
        "outcome": "BLOCKED" if blocked and ok else ("PASS" if ok else "FAIL"),
        "output": output,
    }


def bundle_model(root, variant):
    bundle = root / "bundle.json"
    result = command("docs-bundle", [sys.executable, "scripts/build-docs-bundle.py", str(bundle)])
    result.pop("output")
    if result["outcome"] != "PASS":
        return result
    value = json.loads(bundle.read_text(encoding="utf-8"))
    paths = {item["path"] for item in value["files"]}
    result.update({
        "bundleDocuments": len(paths),
        "curatedSelectionPresent": all(path in paths for path in CURATED),
        "operatorQuickstartPresent": "docs/operations/quickstart/README.md" in paths,
        "variant": variant,
        "journeyModel": "broad bundled browser" if variant == "baseline" else "research-only curated selection",
        "runtimeRoute": "absent",
    })
    bundle.unlink()
    return result


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(root, name, result):
    output = result.pop("output", "")
    (root / f"{name}.txt").write_text(output, encoding="utf-8")
    (root / f"{name}.json").write_text(json.dumps(result, sort_keys=True) + "\n", encoding="utf-8")


def capture(variant):
    raw = Path(tempfile.mkdtemp(prefix="lkjmc-e-product-"))
    results = [
        command("operator-baseline-cli-help", ["cargo", "run", "-p", "lkjmc-cli", "--", "--help"],
                expected=(1,), marker="usage: lkjmc"),
        command("operator-candidate-web", ["./scripts/check-web-smoke.sh"], {"LKJMC_WEB_SMOKE": "1"}),
        command("locale-accessibility-catalog", ["./scripts/check-locales.py"]),
        command("local-menu-containment", ["./scripts/check-jvm-containment.py"]),
        command("adventure-durable-catalog", ["cargo", "test", "-p", "lkjmc-core", "adventure::tests"]),
        command("progression-preflight", ["cargo", "test", "-p", "lkjmc-daemon", "player_shop_tests"]),
        command("guarded-java-menu-protocol", ["./scripts/check-minecraft-smoke.sh"],
                {"LKJMC_MINECRAFT_SMOKE": "1"}, True),
        command("guarded-java-claim", ["./scripts/check-minecraft-claim-smoke.sh"],
                {"LKJMC_MINECRAFT_CLAIM_SMOKE": "1"}, True),
        command("guarded-playable-adventure-transfer", ["./scripts/check-playable-smoke.sh"],
                {"LKJMC_PLAYABLE_SMOKE": "1", "LKJMC_ACCEPT_MINECRAFT_EULA": "1"}, True),
    ]
    results.append(bundle_model(raw, variant))
    for result in results:
        write(raw, result["name"], result)
    summary = {
        "experiment": "E-PRODUCT",
        "base": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "seed": 20260711,
        "variant": variant,
        "results": [{key: value for key, value in item.items() if key != "output"} for item in results],
        "limits": [
            "The curated selection has no registered route.",
            "Java daemon menus, chat fallback, protocol players, claims, adventure, wake, and transfer remain blocked.",
            "CLI, web, locale, containment, and durable-core results are not player journey proof.",
        ],
    }
    (raw / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    files = {path.name: digest(path) for path in sorted(raw.iterdir()) if path.name != "index.json"}
    (raw / "index.json").write_text(json.dumps({"files": files}, indent=2) + "\n", encoding="utf-8")
    failed = [item["name"] for item in results if item["outcome"] == "FAIL"]
    print(f"E-PRODUCT capture={'FAIL' if failed else 'PASS'} variant={variant} raw={raw}")
    print(f"replay=(cd /tmp && python3 {Path(__file__).resolve()} replay --raw-dir {raw})")
    return 1 if failed else 0


def replay(raw):
    root = raw.resolve()
    if root.parent != Path(tempfile.gettempdir()).resolve() or not root.name.startswith("lkjmc-e-product-"):
        print("E-PRODUCT replay=BLOCKED unsafe-root")
        return 2
    try:
        files = json.loads((root / "index.json").read_text(encoding="utf-8"))["files"]
    except (OSError, ValueError, KeyError):
        print("E-PRODUCT replay=BLOCKED index=missing")
        return 2
    bad = [name for name, value in files.items() if not (root / name).is_file() or digest(root / name) != value]
    print("E-PRODUCT replay=" + ("BLOCKED changed=" + ",".join(bad) if bad else "PASS"))
    return 2 if bad else 0


def main():
    parser = argparse.ArgumentParser()
    choices = parser.add_subparsers(dest="action", required=True)
    capture_parser = choices.add_parser("capture")
    capture_parser.add_argument("--variant", choices=("baseline", "candidate"), required=True)
    replay_parser = choices.add_parser("replay")
    replay_parser.add_argument("--raw-dir", type=Path, required=True)
    args = parser.parse_args()
    return capture(args.variant) if args.action == "capture" else replay(args.raw_dir)


if __name__ == "__main__":
    raise SystemExit(main())
