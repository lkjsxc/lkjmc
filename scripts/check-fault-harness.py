#!/usr/bin/env python3
"""Run test-only fault probes and reject their markers from release artifacts."""
import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/execution/fault-seed-replay.json"
PROBES = {
    "failpoints-test-only": "fault_harness",
    "transaction-boundaries-controllable": "transaction_boundaries_are_controllable",
    "effect-boundaries-controllable": "effect_boundaries_are_controllable",
    "deadline-scenario": "deadline_scenario_controls_http_credential_and_shutdown",
    "cross-instance-scenario": "cross_instance_scenario_survives_hang_and_restart",
    "deterministic-seed-replay": "deterministic_seed_replay_reproduces_armed_failure_transcript",
}
MARKERS = (
    b"fault-harness-before-transaction-commit",
    b"fault-harness-after-transaction-commit",
    b"fault-harness-before-process-effect",
    b"fault-harness-after-process-effect",
    b"fault-harness-before-observation",
    b"fault-harness-http-deadline",
    b"fault-harness-credential-lookup",
    b"fault-harness-before-shutdown",
    b"fault-harness-before-jvm-acknowledgement",
)


def run(*command: str) -> None:
    subprocess.run(command, cwd=ROOT, check=True)


def fail(message: str) -> None:
    raise RuntimeError(message)


def require_test_only_sources() -> None:
    daemon = ROOT / "crates/lkjmc-daemon/src"
    main = daemon / "main.rs"
    harness = daemon / "fault_harness.rs"
    control = daemon / "fault_harness/control.rs"
    scenarios = daemon / "fault_harness/scenario_tests.rs"
    if "#[cfg(test)]\nmod fault_harness;" not in main.read_text(encoding="utf-8"):
        fail("fault harness is not gated by cfg(test)")
    if not all(path.is_file() for path in (harness, control, scenarios)):
        fail("missing Rust test-only fault harness sources")
    scenario_text = scenarios.read_text(encoding="utf-8")
    for selector in PROBES.values():
        if selector != "fault_harness" and f"fn {selector}" not in scenario_text:
            fail(f"missing Rust fault scenario: {selector}")
    control_text = control.read_text(encoding="utf-8")
    for marker in MARKERS[:-1]:
        if marker.decode("utf-8") not in control_text:
            fail(f"missing Rust fault marker: {marker.decode('utf-8')}")
    for source in (ROOT / "platforms/jvm").rglob("*.java"):
        if "fault-harness-" in source.read_text(encoding="utf-8"):
            fail(f"Java source contains a Rust fault marker: {source}")
    for source in daemon.rglob("*.rs"):
        if "fault_harness" not in source.parts and "fault-harness-" in source.read_text(encoding="utf-8"):
            fail(f"release Rust source contains a fault marker: {source}")


def require_markers_absent(content: bytes, target: Path) -> None:
    if any(marker in content for marker in MARKERS):
        fail(f"release artifact contains a test failpoint marker: {target}")


def release_inspection() -> None:
    require_test_only_sources()
    run("cargo", "build", "--release", "-p", "lkjmc-daemon")
    daemon = ROOT / "target/release/lkjmc-daemon"
    require_markers_absent(daemon.read_bytes(), daemon)


def rust_probe(name: str, capture: bool = False) -> str:
    command = ["cargo", "test", "-p", "lkjmc-daemon", PROBES[name]]
    if capture:
        command.extend(["--", "--nocapture"])
        result = subprocess.run(command, cwd=ROOT, check=True, text=True, capture_output=True)
        print(result.stdout, end="")
        print(result.stderr, end="", file=sys.stderr)
        return result.stdout + result.stderr
    run(*command)
    return ""


def rust_list(values: list[str]) -> str:
    return "[" + ", ".join(f'\"{value}\"' for value in values) + "]"


def expected_transcript(item: dict[str, object]) -> str:
    state = item["state"]
    if not isinstance(state, dict):
        fail("seed evidence state must be an object")
    hits = ", ".join(item["hits"])
    return (
        f"FailureTranscript {{ seed: {item['seed']}, clock_ms: {item['clockMs']}, "
        f"boundary: {item['boundary']}, hits: [{hits}], state: ScenarioState {{ "
        f"order: {rust_list(state['order'])}, started: {rust_list(state['started'])}, "
        f"observed: {rust_list(state['observed'])} }} }}"
    )


def replay_evidence() -> None:
    try:
        evidence = json.loads(EVIDENCE.read_text(encoding="utf-8"))
        replay = evidence["replay"]
        same_seed = replay["sameSeed"]
        different_seed = replay["differentSeed"]
        review = evidence["qualityReview"]
    except (OSError, KeyError, TypeError, json.JSONDecodeError) as error:
        fail(f"invalid seed failure evidence: {error}")
    if not all(isinstance(item, dict) for item in (replay, same_seed, different_seed, review)):
        fail("seed evidence sections must be objects")
    if replay.get("selector") != "deterministic-seed-replay" or review.get("status") != "pending":
        fail("seed evidence must name the replay selector and pending review")
    if same_seed.get("outcome") != "Err" or different_seed.get("outcome") != "Err":
        fail("seed evidence must record injected failures")
    if same_seed.get("seed") == different_seed.get("seed"):
        fail("seed evidence must compare different seeds")
    if same_seed.get("state", {}).get("order") == different_seed.get("state", {}).get("order"):
        fail("seed evidence must record distinguishable orders")
    output = rust_probe("deterministic-seed-replay", capture=True)
    if f"seed-failure-replay={expected_transcript(same_seed)}" not in output:
        fail("seed replay output does not match the recorded same-seed transcript")
    if f"seed-failure-different={expected_transcript(different_seed)}" not in output:
        fail("seed replay output does not match the recorded different-seed transcript")


def selected_probe(name: str) -> None:
    if name == "deterministic-seed-replay":
        replay_evidence()
    else:
        rust_probe(name)
    if name == "failpoints-test-only":
        release_inspection()
    print(f"ok {name}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--probe", choices=sorted(PROBES))
    parser.add_argument("--all", action="store_true")
    args = parser.parse_args()
    try:
        if args.all:
            for name in PROBES:
                selected_probe(name)
        elif args.probe:
            selected_probe(args.probe)
        else:
            parser.error("choose --all or --probe")
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"fault harness failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
