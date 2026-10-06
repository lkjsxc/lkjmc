#!/usr/bin/env python3
"""Retain only bounded, redacted assertion logs after a failed CI acceptance."""
import json
import math
import os
from pathlib import Path
import re
import stat
import subprocess

MAX_REPORT = 1024 * 1024
MAX_TAIL = 32 * 1024
MAX_FILES = 20
MAX_SECRET = 64 * 1024
MAX_RUNS = 8
OUTER_LOGS = ("browser-fixture.log", "browser-integration.log", "game-protocol.log")
FIXTURE_LOGS = ("browser-integration.log", "game-protocol.log", "game-expeditions.log",
                "game-setup.log", "expedition-setup.log")
NETWORK_ROLES = ("official", "lobby", "proxy")
SECRET_FILES = (".local/browser-session.json", ".local/game/official-token",
                ".local/game/lobby-token", ".local/game/proxy-token",
                ".local/game/forwarding-secret", ".local/game/departure-key")
SENSITIVE = re.compile(r"token|secret|password|passwd|credential|cookie|authorization|api[_-]?key|private[_-]?key|csrf|session|database_url", re.I)
HEADERS = re.compile(r"(?im)\b(?:authorization|proxy-authorization|cookie|set-cookie)[\"']?\s*[:=]\s*[^\r\n]*")
ASSIGNMENTS = re.compile(r"(?i)([\"']?\b[a-z0-9_-]*(?:token|secret|password|passwd|credential|api[_-]?key|private[_-]?key|csrf|session)[a-z0-9_-]*[\"']?\s*[:=]\s*)(?:\"(?:\\.|[^\"\\\r\n])*\"|'(?:\\.|[^'\\\r\n])*'|[^\s,;}]+)")
PRIVATE_KEY = re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----", re.S)
AUTH_URI = re.compile(r"\b((?:postgres(?:ql)?|mysql|redis|mongodb)://)[^\s/@]+@", re.I)


def open_relative(root_fd, relative, directory=False):
    """Open every component beneath root with NOFOLLOW, including ancestors."""
    parts = Path(relative).parts
    if not parts or Path(relative).is_absolute() or any(p in (".", "..") for p in parts):
        raise ValueError("invalid relative diagnostic path")
    current = os.dup(root_fd)
    try:
        for part in parts[:-1]:
            next_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=current)
            os.close(current)
            current = next_fd
        flags = os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK
        if directory:
            flags |= os.O_DIRECTORY
        result = os.open(parts[-1], flags, dir_fd=current)
        info = os.fstat(result)
        if not directory and (not stat.S_ISREG(info.st_mode) or info.st_nlink != 1):
            os.close(result)
            raise ValueError("diagnostic must be a regular, single-link file")
        return result
    finally:
        os.close(current)


def read_small(root_fd, relative):
    try:
        fd = open_relative(root_fd, relative)
    except FileNotFoundError:
        return None
    with os.fdopen(fd, "rb") as source:
        if os.fstat(source.fileno()).st_size > MAX_SECRET:
            raise ValueError("oversized receipt or redaction source")
        return source.read(MAX_SECRET).decode("utf-8", errors="replace")


def fixture_runs(root_fd):
    try:
        fd = open_relative(root_fd, ".local/ux", directory=True)
    except OSError:
        return []
    runs = []
    try:
        with os.scandir(fd) as entries:
            for index, entry in enumerate(entries):
                if index >= 256:
                    break
                if re.fullmatch(r"real-[0-9a-f]{12}", entry.name) and entry.is_dir(follow_symlinks=False):
                    runs.append((entry.stat(follow_symlinks=False).st_mtime_ns, entry.name))
    finally:
        os.close(fd)
    return [".local/ux/" + name for _, name in sorted(runs, reverse=True)[:MAX_RUNS]]


def network_logs(root_fd, runs, acceptance):
    """Keep only the exact network fixture started within this failed CI job."""
    if acceptance is None:
        return []
    started, finished = acceptance.get("started"), acceptance.get("finished")
    if any(type(value) not in (int, float) or (type(value) is float and not math.isfinite(value))
           for value in (started, finished)) or started > finished:
        return []
    for run in runs:
        try:
            raw = read_small(root_fd, run + "/network-logs.json")
            if raw is None:
                continue
            manifest = json.loads(raw)
            if not isinstance(manifest, dict) or type(manifest.get("schema")) is not int or manifest["schema"] != 1:
                continue
            if manifest.get("fixture_id") != run.rsplit("real-", 1)[1]:
                continue
            tag = manifest.get("network_id")
            when = manifest.get("started_at")
            if not isinstance(tag, str) or not re.fullmatch(r"[0-9a-f]{6}", tag):
                continue
            if type(when) not in (int, float) or (type(when) is float and not math.isfinite(when)) or not started <= when <= finished:
                continue
            logs = [f".local/game/network-{tag}-{role}.log" for role in NETWORK_ROLES]
            if manifest.get("logs") == logs:
                return logs
        except (OSError, ValueError):
            continue
    return []


def session_values(value):
    if isinstance(value, dict):
        if SENSITIVE.search(str(value.get("name", ""))) and isinstance(value.get("value"), str):
            yield value["value"]
        for key, item in value.items():
            if SENSITIVE.search(key) and isinstance(item, str):
                yield item
            elif key == "cookies" and isinstance(item, list):
                for cookie in item:
                    if isinstance(cookie, dict) and isinstance(cookie.get("value"), str):
                        yield cookie["value"]
            else:
                yield from session_values(item)
    elif isinstance(value, list):
        for item in value:
            yield from session_values(item)


def secrets_for(root_fd, runs, environment):
    values = {value for name, value in environment.items() if SENSITIVE.search(name) and value}
    for relative in (*SECRET_FILES, *(run + "/browser-session.json" for run in runs)):
        raw = read_small(root_fd, relative)
        if raw is None:
            continue
        if relative.endswith(".json"):
            try:
                values.update(value for value in session_values(json.loads(raw)) if value)
            except json.JSONDecodeError as error:
                raise ValueError("invalid fixture redaction source") from error
        elif raw.strip():
            values.add(raw.strip())
    if any(len(value.encode("utf-8")) > MAX_SECRET for value in values):
        raise ValueError("oversized environment redaction value")
    # Also recognize the representation of a known value in JSON log output.
    values.update(json.dumps(value)[1:-1] for value in tuple(values))
    return sorted(values, key=len, reverse=True)


def redact(text, secrets, root):
    for secret in secrets:
        text = text.replace(secret, "[REDACTED]")
    text = PRIVATE_KEY.sub("[REDACTED PRIVATE KEY]", text)
    text = HEADERS.sub("[REDACTED AUTHENTICATION HEADER]", text)
    text = ASSIGNMENTS.sub(lambda match: match[1] + "[REDACTED]", text)
    text = AUTH_URI.sub(r"\1[REDACTED]@", text)
    return text.replace(str(root), "$WORKSPACE")


def log_tail(root_fd, relative, secrets, root):
    fd = open_relative(root_fd, relative)
    with os.fdopen(fd, "rb") as source:
        size = os.fstat(source.fileno()).st_size
        overlap = max((len(value.encode("utf-8")) for value in secrets), default=0)
        offset = max(0, size - MAX_TAIL - overlap - 1)
        source.seek(offset)
        raw = source.read(MAX_TAIL + overlap + 1)
    # Discard a partial leading line so a clipped credential assignment cannot
    # evade redaction. Complete known values are redacted before final clipping.
    if offset:
        raw = raw.partition(b"\n")[2]
    text = redact(raw.decode("utf-8", errors="replace"), secrets, root)
    encoded = text.encode("utf-8")
    return {"path": relative, "source_bytes": size,
            "tail": encoded[-MAX_TAIL:].decode("utf-8", errors="ignore"),
            "truncated": bool(offset or len(encoded) > MAX_TAIL)}


def collect(root, commit, environment):
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise ValueError("a full source commit is required")
    root = Path(root).absolute()
    root_fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        receipt = read_small(root_fd, ".local/ci/reports/acceptance.json")
        acceptance = None
        if receipt is not None:
            acceptance = json.loads(receipt)
            if acceptance.get("commit") != commit or acceptance.get("status") != "failed":
                raise ValueError("acceptance must identify the same failed commit")
        runs = fixture_runs(root_fd)
        secrets = secrets_for(root_fd, runs, environment)
        report = {"schema": 1, "commit": commit, "failure_only": True,
                  "acceptance_receipt_present": receipt is not None,
                  "max_bytes": MAX_REPORT, "files": [], "omitted": 0}
        candidates = [".local/ci/private/" + name for name in OUTER_LOGS]
        # Raw logs retain stack-trace continuation lines filtered from the
        # protocol transcript. Prioritize them before older fixture transcripts.
        candidates += network_logs(root_fd, runs, acceptance)
        candidates += [run + "/" + name for run in runs for name in FIXTURE_LOGS]
        for relative in candidates:
            try:
                entry = log_tail(root_fd, relative, secrets, root)
            except (OSError, ValueError):
                continue
            if len(report["files"]) >= MAX_FILES:
                report["omitted"] += 1
                continue
            report["files"].append(entry)
        # JSON escaping can expand unusual log bytes. Enforce the serialized
        # byte limit, not just a per-file character limit.
        while len(serialize(report)) > MAX_REPORT and report["files"]:
            report["files"].pop()
            report["omitted"] += 1
        return report
    finally:
        os.close(root_fd)


def serialize(report):
    return (json.dumps(report, ensure_ascii=False, separators=(",", ":")) + "\n").encode("utf-8")


def write_report(root, report):
    root_fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    current = root_fd
    try:
        for part in (".local", "ci", "reports"):
            try:
                os.mkdir(part, mode=0o700, dir_fd=current)
            except FileExistsError:
                pass
            next_fd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=current)
            if current != root_fd:
                os.close(current)
            current = next_fd
        fd = os.open("failure-diagnostics.json", os.O_WRONLY | os.O_CREAT | os.O_NOFOLLOW | os.O_NONBLOCK,
                     mode=0o600, dir_fd=current)
        with os.fdopen(fd, "wb") as output:
            info = os.fstat(output.fileno())
            if not stat.S_ISREG(info.st_mode) or info.st_nlink != 1:
                raise ValueError("diagnostic output must be a regular, single-link file")
            os.fchmod(output.fileno(), 0o600)
            output.truncate(0)
            output.write(serialize(report))
    finally:
        if current != root_fd:
            os.close(current)
        os.close(root_fd)


def main():
    root = Path.cwd()
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    if commit != os.environ.get("CI_COMMIT"):
        raise ValueError("CI_COMMIT must match the checked-out source")
    report = collect(root, commit, os.environ)
    write_report(root, report)
    print(json.dumps({"commit": commit, "diagnostic_files": len(report["files"]),
                      "bytes": len(serialize(report)), "omitted": report["omitted"]}))


if __name__ == "__main__":
    main()
