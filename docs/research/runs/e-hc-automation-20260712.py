#!/usr/bin/env python3
"""Replay-only evidence for E-HC-AUTOMATION; it never calls product commands."""
from __future__ import annotations
import argparse, hashlib, json, math, re, secrets, shutil, subprocess, tempfile, time
from pathlib import Path

SCRIPT = Path(__file__).resolve()
ROOT = Path(subprocess.check_output(["git", "-C", str(SCRIPT.parent), "rev-parse", "--show-toplevel"], text=True).strip()).resolve()
FIXTURE = SCRIPT.with_name("e-hc-automation-20260712-fixtures.json")
TEMP = Path(tempfile.gettempdir()).resolve()
RAW_RE = re.compile(r"lkjmc-e-hc-automation-[0-9a-f]{32}$")
LIMIT = 8192
ACTIONS = {
    "daemon-unavailable": "stop-dependent-mutations-and-inspect",
    "database-error": "hold-writes-and-inspect",
    "backend-unavailable": "hold-transfers-and-inspect",
    "suspected-secret-exposure": "revoke-access-and-inspect",
}


def raw_dir(value: Path | None) -> Path:
    raw = value or TEMP / ("lkjmc-e-hc-automation-" + secrets.token_hex(16))
    raw = raw.resolve()
    if raw.parent != TEMP or not RAW_RE.fullmatch(raw.name) or raw.exists():
        raise ValueError("raw root must be a new /tmp/lkjmc-e-hc-automation-<32 hex> directory")
    raw.mkdir(mode=0o700)
    (raw / ".owned").write_text(raw.name + "\n", encoding="utf-8")
    return raw


def write(raw: Path, name: str, text: str, artifacts: list[dict]) -> None:
    data = text.encode("utf-8")[-LIMIT:]
    (raw / name).write_bytes(data)
    artifacts.append({"path": name, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()})


def command(raw: Path, artifacts: list[dict], name: str, args: list[str], timeout: int = 120) -> tuple[int, str]:
    started = time.monotonic()
    try:
        done = subprocess.run(args, cwd=ROOT, text=True, stdout=subprocess.PIPE,
                              stderr=subprocess.STDOUT, timeout=timeout, check=False)
        code, output = done.returncode, done.stdout
    except (FileNotFoundError, subprocess.TimeoutExpired) as error:
        code, output = 127 if isinstance(error, FileNotFoundError) else 124, str(error)
    write(raw, name + ".log", f"exit={code} seconds={time.monotonic()-started:.3f}\n{output}", artifacts)
    return code, output


def percentile(values: list[int]) -> int | None:
    return sorted(values)[math.ceil(len(values) * .95) - 1] if values else None


def offline(data: dict) -> dict:
    recommendations = [{"id": row["id"], "disposition": "operator-review",
                        "recommendation": ACTIONS.get(row["symptom"], "no-recommendation")}
                       for row in data["incidents"]]
    windows = data["wakeWindows"]
    eligible = [row for row in windows if row["presence"] == "known-empty"]
    joins = [row for row in windows if row["joinAt"]]
    hits = [row for row in eligible if row["joinAt"]]
    regions = {}
    for row in data["regionAttempts"]:
        region = regions.setdefault(row["region"], {"attempts": 0, "failures": 0, "latenciesMs": []})
        region["attempts"] += 1
        if row["outcome"] == "ok": region["latenciesMs"].append(row["latencyMs"])
        else: region["failures"] += 1
    for region in regions.values(): region["p95Ms"] = percentile(region.pop("latenciesMs"))
    return {
        "incidentRecommendations": recommendations,
        "wakeReplay": {"reactiveJoinDelaySeconds": sum(row["wakeSeconds"] for row in joins),
                       "predictiveJoinDelaySeconds": sum(row["wakeSeconds"] for row in joins if row not in hits),
                       "eligiblePredictions": len(eligible), "hits": len(hits),
                       "falsePrewarms": len([row for row in eligible if not row["joinAt"]]),
                       "unknownPresenceSkipped": len([row for row in windows if row["presence"] != "known-empty"])},
        "regionModel": regions,
    }


def postgres(raw: Path, artifacts: list[dict], data: dict) -> dict:
    image = "postgres:16-alpine"
    prerequisite = "Docker/Compose and a locally present postgres:16-alpine image"
    if shutil.which("docker") is None:
        return {"name": "real-postgresql", "state": "BLOCKED", "reason": "docker is unavailable", "prerequisite": prerequisite}
    present, output = command(raw, artifacts, "postgres-image-preflight", ["docker", "image", "inspect", image])
    if present:
        reason = "local postgres:16-alpine image is absent; Compose was not started"
        if "No such object" not in output:
            reason = "local postgres:16-alpine image preflight failed; Compose was not started"
        return {"name": "real-postgresql", "state": "BLOCKED", "reason": reason, "prerequisite": prerequisite}
    project = "lkjmchc" + secrets.token_hex(5)
    compose = ["docker", "compose", "--project-name", project, "-f", str(ROOT / "docker-compose.yml")]
    up, _ = command(raw, artifacts, "postgres-up", [*compose, "up", "-d", "--wait", "--pull", "never", "postgres"], 240)
    state, reason = "BLOCKED", "Compose PostgreSQL did not start with --pull never"
    try:
        rows = []
        for row in data["regionAttempts"]:
            latency = "NULL" if row["latencyMs"] is None else str(row["latencyMs"])
            rows.append(f"('{row['region']}',{latency},'{row['outcome']}')")
        sql = "drop table if exists hc_automation_replay; create table hc_automation_replay(region text, latency_ms int, outcome text); insert into hc_automation_replay values " + ",".join(rows) + "; select region,count(*),count(*) filter (where outcome <> 'ok') from hc_automation_replay group by region order by region;"
        query, output = command(raw, artifacts, "postgres-model", [*compose, "exec", "-T", "postgres", "psql", "-v", "ON_ERROR_STOP=1", "-At", "-F", "|", "-U", "lkjmc", "-d", "lkjmc", "-c", sql])
        delay, _ = command(raw, artifacts, "postgres-delay", [*compose, "exec", "-T", "postgres", "psql", "-v", "ON_ERROR_STOP=1", "-At", "-U", "lkjmc", "-d", "lkjmc", "-c", "select floor(extract(epoch from clock_timestamp()-statement_timestamp())*1000)::int from (select pg_sleep(.075)) as delayed;"])
        timeout, timed = command(raw, artifacts, "postgres-timeout", [*compose, "exec", "-T", "postgres", "psql", "-v", "ON_ERROR_STOP=1", "-U", "lkjmc", "-d", "lkjmc", "-c", "set statement_timeout='25ms'; select pg_sleep(.075);"])
        expected = all(value in output for value in ("same-region|3|0", "near-region|3|1", "far-region|2|1"))
        if not up and not query and not delay and timeout and "statement timeout" in timed and expected:
            state, reason = "PASS", "disposable PostgreSQL loaded the replay and observed delay plus statement-timeout failure"
        else: reason = "PostgreSQL model, delay, or expected timeout was not observed"
    finally:
        down, _ = command(raw, artifacts, "postgres-down", [*compose, "down", "--volumes", "--remove-orphans"], 120)
        if down: state, reason = "BLOCKED", "Compose cleanup did not complete"
    return {"name": "real-postgresql", "state": state, "reason": reason, "prerequisite": prerequisite}


def netem(raw: Path, artifacts: list[dict]) -> dict:
    prerequisite = "Linux user/network namespaces plus iproute2 tc netem; an authorized remote endpoint is separate"
    args = ["unshare", "--user", "--map-root-user", "--net", "sh", "-ec", "ip link set lo up; tc qdisc replace dev lo root netem delay 75ms; tc qdisc show dev lo; tc qdisc del dev lo root"]
    code, _ = command(raw, artifacts, "isolated-netem", args)
    state = "PASS" if code == 0 else "BLOCKED"
    reason = "isolated netem configured and removed; it is not a remote-region proof" if not code else "isolated namespace or tc netem was unavailable or denied"
    return {"name": "network-shaping", "state": state, "reason": reason, "prerequisite": prerequisite, "rerun": " ".join(args)}


def run(args: argparse.Namespace) -> int:
    raw, artifacts = raw_dir(args.raw_dir), []
    data = json.loads(FIXTURE.read_text(encoding="utf-8"))
    model = offline(data)
    write(raw, "fixture.json", json.dumps(data, indent=2, sort_keys=True) + "\n", artifacts)
    write(raw, "offline-model.json", json.dumps(model, indent=2, sort_keys=True) + "\n", artifacts)
    lanes = [postgres(raw, artifacts, data), netem(raw, artifacts)]
    coverage = {"fixture": "fixture.json", "offlineModel": "offline-model.json",
                "commands": [item["path"] for item in artifacts if item["path"].endswith(".log")]}
    index = {"format": "e-hc-automation-v2", "base": "4b9357a8e1a7949e0ebfe59c16af5196554f46cc", "fixtureSha256": hashlib.sha256(FIXTURE.read_bytes()).hexdigest(), "model": model, "lanes": lanes, "coverage": coverage, "artifacts": artifacts}
    encoded = json.dumps(index, indent=2, sort_keys=True) + "\n"
    (raw / "index.json").write_text(encoded, encoding="utf-8")
    digest = hashlib.sha256(encoded.encode()).hexdigest()
    (raw / "index.sha256").write_text(digest + "  index.json\n", encoding="utf-8")
    print(f"E-HC-AUTOMATION raw={raw} sha256={digest}")
    print(" ".join(f"{item['name']}={item['state']}" for item in lanes))
    return 0


def replay(args: argparse.Namespace) -> int:
    try:
        raw = args.raw_dir.resolve()
        index = json.loads((raw / "index.json").read_text(encoding="utf-8"))
        valid = (raw / ".owned").read_text(encoding="utf-8") == raw.name + "\n"
        valid &= hashlib.sha256((raw / "index.json").read_bytes()).hexdigest() == (raw / "index.sha256").read_text().split()[0]
        for item in index["artifacts"]:
            path = raw / item["path"]
            valid &= path.is_file() and hashlib.sha256(path.read_bytes()).hexdigest() == item["sha256"]
        coverage = index["coverage"]
        valid &= index["format"] == "e-hc-automation-v2"
        valid &= index["fixtureSha256"] == hashlib.sha256(FIXTURE.read_bytes()).hexdigest()
        valid &= json.loads((raw / coverage["fixture"]).read_text()) == json.loads(FIXTURE.read_text())
        valid &= json.loads((raw / coverage["offlineModel"]).read_text()) == index["model"]
        names = {item["path"] for item in index["artifacts"]}
        required = {coverage["fixture"], coverage["offlineModel"], "postgres-image-preflight.log", "isolated-netem.log"}
        valid &= required.issubset(names)
        valid &= set(coverage["commands"]) == {name for name in names if name.endswith(".log")}
    except (OSError, ValueError, KeyError, json.JSONDecodeError, IndexError): valid = False
    print("E-HC-AUTOMATION replay=" + ("PASS" if valid else "BLOCKED"))
    return int(not valid)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=["run", "replay"])
    parser.add_argument("--raw-dir", type=Path)
    args = parser.parse_args()
    return run(args) if args.action == "run" else replay(args)


if __name__ == "__main__":
    raise SystemExit(main())
