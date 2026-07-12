#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DAEMON = ROOT / "crates/lkjmc-daemon/src"


def text(path):
    return path.read_text(encoding="utf-8")


def contracts():
    for path in (ROOT / "contracts/commands").glob("*.json"):
        if path.name != "README.json":
            yield from json.loads(text(path))["commands"]


def forgery_suite_pass():
    authz = text(DAEMON / "authz.rs")
    tests = text(DAEMON / "authz_tests.rs")
    return "request.body" not in authz and "credential_replaces_untrusted" in tests


def surface_policy_complete():
    known = {"admin", "operator", "player"}
    surfaces = {"internal", "cli", "web"}
    return all(
        command["authorization"] in known
        and set(command["surfaces"]) <= surfaces
        for command in contracts()
    ) and ".surfaces" in text(DAEMON / "authz.rs")


def revocation_bound_pass():
    cache = text(DAEMON / "credential_cache.rs")
    migration = text(ROOT / "migrations/043-daemon-token-revision.sql")
    return (
        cache.index("current_revision") < cache.index("self.cached")
        and "state.entries.clear()" in cache
        and "pg_notify('lkjmc_daemon_token_revision'" in migration
    )


def reactor_blocking_absent():
    auth = text(DAEMON / "transport/auth.rs")
    peer = text(DAEMON / "transport/peer.rs")
    return (
        "spawn_blocking" in auth
        and "database_connection" not in auth
        and "spawn_blocking" in peer
    )


def root_routine_absent():
    sources = [
        DAEMON / "authz.rs",
        DAEMON / "transport/auth.rs",
        DAEMON / "transport/command.rs",
        DAEMON / "web/api.rs",
    ]
    return all("AuthenticatedSubject::root" not in text(path) for path in sources)


def web_csrf_session_pass():
    auth = text(DAEMON / "web/auth.rs")
    sessions = text(DAEMON / "web/sessions.rs")
    return all(
        value in auth + sessions
        for value in ["csrf_allowed", "MAX_LOGIN_ATTEMPTS", "MAX_LOGIN_SOURCES", "verify("]
    )


def secret_canary_pass():
    provider = text(DAEMON / "support/secret_provider.rs")
    audit = text(DAEMON / "security_audit.rs")
    return "security-canary" in provider and 'target_id: "redacted"' in audit


PROBES = {
    "forgery-suite-pass": forgery_suite_pass,
    "surface-policy-complete": surface_policy_complete,
    "revocation-bound-pass": revocation_bound_pass,
    "reactor-blocking-absent": reactor_blocking_absent,
    "root-routine-absent": root_routine_absent,
    "web-csrf-session-pass": web_csrf_session_pass,
    "secret-canary-pass": secret_canary_pass,
}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--probe", choices=sorted(PROBES))
    args = parser.parse_args()
    names = [args.probe] if args.probe else sorted(PROBES)
    failures = [name for name in names if not PROBES[name]()]
    for name in names:
        print(("failed " if name in failures else "ok ") + name)
    return bool(failures)


if __name__ == "__main__":
    raise SystemExit(main())
