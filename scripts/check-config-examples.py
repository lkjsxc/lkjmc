#!/usr/bin/env python3
"""Validate JSON examples through the production Rust parser."""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CLI = ["cargo", "run", "--quiet", "-p", "lkjmc-cli", "--", "config", "check", "--path"]


def parse(path: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [*CLI, str(path)], cwd=ROOT, text=True, capture_output=True, check=False
    )


def error_output(result: subprocess.CompletedProcess[str]) -> str:
    return (result.stderr or result.stdout).strip()


def main() -> int:
    files = sorted((ROOT / "config/defaults").glob("*.json.example"))
    errors: list[str] = []
    if not files:
        errors.append("config examples: no JSON examples found")
    for path in files:
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            errors.append(f"{path}: invalid JSON: {error}")
            continue
        result = parse(path)
        if result.returncode:
            errors.append(f"{path}: Rust parser rejected example: {error_output(result)}")
            continue
        data["database"]["poolSize"] = 0
        with tempfile.TemporaryDirectory(prefix="lkjmc-config-check-") as directory:
            invalid = Path(directory) / "invalid.json"
            invalid.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
            rejected = parse(invalid)
        if not rejected.returncode or "database.poolSize" not in error_output(rejected):
            errors.append(f"{path}: Rust parser accepted invalid poolSize")
    if errors:
        print("\n".join(errors))
        return 1
    print("ok check-config-examples parser=rust")
    return 0


if __name__ == "__main__":
    sys.exit(main())
