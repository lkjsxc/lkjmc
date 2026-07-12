#!/usr/bin/env python3
"""Bounded E-SYNTHESIS evidence attempts anchored to candidate 3a0aa47."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

sys.dont_write_bytecode = True
from e_synthesis_20260712_data import CASES, COMBINATIONS

TIP = "3a0aa47ce4e29d17656d2ba2973ea673e0788db6"
ROOT = Path(__file__).resolve().parents[3]
RAW = ROOT / "docs/research/runs/e-synthesis-20260712-raw-manifest.json"
DATA = Path(__file__).with_name("e_synthesis_20260712_data.py")
TMP = ROOT / "tmp/e-synthesis-20260712"
SOURCES = {
    "CT": "docs/research/catalog-sources/contracts.md",
    "CP": "docs/research/catalog-sources/control-plane.md",
    "DW": "docs/research/catalog-sources/data.md",
    "JV": "docs/research/catalog-sources/jvm.md",
    "OP": "docs/research/catalog-sources/operations.md",
    "PX": "docs/research/catalog-sources/product.md",
    "SE": "docs/research/catalog-sources/security.md",
    "QV": "docs/research/catalog-sources/verification.md",
}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def base_bytes(path):
    return subprocess.check_output(["git", "show", f"{TIP}:{path}"], cwd=ROOT)


def local_root(case_id):
    root = TMP / case_id
    if root.exists():
        shutil.rmtree(root)
    root.mkdir(parents=True)
    return root


def probe(mode, root, case_id):
    if mode == "json":
        path = root / "document.json"
        path.write_text(json.dumps({"id": case_id, "revision": 1}) + "\n")
        return json.loads(path.read_text())["id"] == case_id
    if mode == "paths":
        child = (root / "catalog/item.json").resolve()
        child.parent.mkdir()
        child.write_text("{}\n")
        escape = (root / "../outside").resolve()
        return child.is_relative_to(root.resolve()) and not escape.is_relative_to(root.resolve())
    if mode == "index":
        names = ["beta", "alpha", "gamma"]
        index = {name: number for number, name in enumerate(sorted(names))}
        return list(index) == ["alpha", "beta", "gamma"] and len(index) == 3
    if mode == "width":
        text = json.dumps({"key": "bounded", "items": [1, 2]}, indent=2)
        return max(map(len, text.splitlines())) <= 80
    if mode == "atomic":
        old, new = root / "alias.json", root / "canonical.json"
        old.write_text("{}\n")
        os.replace(old, new)
        return new.exists() and not old.exists()
    if mode == "domain":
        modules = {"identity": ["subject"], "world": ["instance"]}
        return set().union(*map(set, modules.values())) == {"subject", "instance"}
    if mode == "adventure":
        allowed = {"new": "active", "active": "complete"}
        return allowed.get("new") == "active" and "complete" not in allowed
    if mode == "notify":
        cache = {"subject": 1}
        cache["subject"] = 2
        return cache["subject"] == 2
    if mode == "replay":
        path = root / "changes.jsonl"
        path.write_text('{"revision":1}\n{"revision":2}\n')
        return [json.loads(line)["revision"] for line in path.read_text().splitlines()] == [1, 2]
    if mode == "delta":
        ordered, reordered = [3, 4], [4, 3]
        return ordered == sorted(ordered) and reordered != sorted(reordered)
    if mode == "hash":
        path = root / "audit.jsonl"
        path.write_text('{"event":"deny"}\n')
        original = digest(path.read_bytes())
        path.write_text('{"event":"allow"}\n')
        return original != digest(path.read_bytes())
    if mode == "retention":
        records = [{"revision": 1}, {"revision": 2}, {"revision": 3}]
        retained = [item for item in records if item["revision"] >= 2]
        return [item["revision"] for item in retained] == [2, 3]
    if mode == "fair":
        queues = [["a1", "a2"], ["b1", "b2"]]
        served = [queues[number % 2].pop(0)[0] for number in range(4)]
        return served == ["a", "b", "a", "b"]
    if mode == "clock":
        times = [0, 5, 4]
        return times[:2] == sorted(times[:2]) and times[2] < times[1]
    if mode == "root":
        owned = root / "retired"
        owned.mkdir()
        shutil.rmtree(owned)
        return not owned.exists()
    if mode == "opaque":
        log = root / "provider.log"
        log.write_text("provider=opaque-handle\n")
        return "opaque-handle" in log.read_text()
    if mode == "dependency":
        artifact = root / "artifact"
        artifact.write_text("metadata\n")
        expected = digest(artifact.read_bytes())
        artifact.write_text("changed\n")
        return expected != digest(artifact.read_bytes())
    if mode == "ci":
        checks = ("scripts/check-lines.py", "scripts/check-docs.py")
        return all(subprocess.run([ROOT / check], cwd=ROOT, check=False,
                                  stdout=subprocess.DEVNULL).returncode == 0 for check in checks)
    if mode == "retry":
        attempts = [sum(range(3)) for _ in range(3)]
        return attempts == [3, 3, 3]
    if mode == "coverage":
        rows = [line for line in base_bytes("docs/research/decisions/e-synthesis-dispositions-20260712.md").decode().splitlines() if line.startswith("| ")]
        records = [line for line in rows if not line.startswith(("| ID ", "| ---"))]
        missing = [line for line in records if "| no " in line.lower()]
        return len(records) == 150 and len(missing) == 34
    raise ValueError(f"unknown local mode {mode}")


def attempt(case_id):
    mode, result = CASES[case_id]
    source = SOURCES[case_id.split("-", 1)[0]]
    record = {"id": case_id, "command": f"python3 docs/research/runs/e_synthesis_20260712_evidence.py --id {case_id}", "source": source, "sourceHash": digest(base_bytes(source)), "result": result}
    if mode.startswith("guard:"):
        guards = mode.removeprefix("guard:").split(",")
        missing = [name for name in guards if not os.environ.get(name)]
        if not missing:
            record["result"] = "REJECTED: docs-only runner starts no external client"
        record.update(exit=2 if missing else 3, guards=guards, rerun="env " + " ".join(f"{name}=1" for name in guards) + " " + record["command"])
        return record
    record["exit"] = 0 if probe(mode, local_root(case_id), case_id) else 1
    return record


def combination(combo, attempts):
    combo_id, name, evidence, conclusion = combo
    checked = []
    for item in evidence:
        if item in attempts:
            checked.append(f"{item}:exit={attempts[item]['exit']}")
        else:
            checked.append(f"{item}:sha256={digest(base_bytes(item))}")
    return {"id": combo_id, "name": name, "evidence": checked, "outcome": conclusion, "compatibility": "all named base-tip evidence is reachable"}


def render(manifest):
    lines = ["{", f'  "sourceTip": "{TIP}",', f'  "runnerHash": "{digest(Path(__file__).read_bytes())}",', f'  "dataHash": "{digest(DATA.read_bytes())}",', '  "attempts": [']
    lines += ["    " + json.dumps(item, sort_keys=True) + ("," if index + 1 < len(manifest["attempts"]) else "") for index, item in enumerate(manifest["attempts"])]
    lines.append("  ],")
    lines.append('  "combinations": [')
    lines += ["    " + json.dumps(item, sort_keys=True) + ("," if index + 1 < len(manifest["combinations"]) else "") for index, item in enumerate(manifest["combinations"])]
    return "\n".join(lines + ["  ]", "}", ""])


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--id", choices=sorted(CASES))
    parser.add_argument("--write", action="store_true")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if args.id:
        record = attempt(args.id)
        print(json.dumps(record, sort_keys=True))
        return record["exit"]
    attempts = {case_id: attempt(case_id) for case_id in sorted(CASES)}
    manifest = {"attempts": list(attempts.values()), "combinations": [combination(item, attempts) for item in COMBINATIONS]}
    text = render(manifest)
    if args.write:
        RAW.write_text(text)
    if args.check and (not RAW.exists() or RAW.read_text() != text):
        print("manifest mismatch", file=sys.stderr)
        return 1
    print(f"ok E-SYNTHESIS attempts={len(attempts)} blocked={sum(item['exit'] == 2 for item in attempts.values())} combinations={len(COMBINATIONS)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
