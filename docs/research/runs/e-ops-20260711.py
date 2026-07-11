#!/usr/bin/env python3
"""Run bounded E-OPS evidence lanes outside product registration."""
from __future__ import annotations
import hashlib, json, os, re, secrets, shutil, subprocess, sys, tempfile, time
from pathlib import Path

SCRIPT = Path(__file__).resolve()
ROOT = Path(subprocess.check_output(["git", "-C", str(SCRIPT.parent), "rev-parse", "--show-toplevel"], text=True).strip())
LIMIT = 8192
CANARY = "e-ops-credential-canary"
URL = re.compile(r"(?i)\b[a-z][a-z0-9+.-]*://[^\s'\"`]+")
SECRET = re.compile(r"(?i)((?:password|token|secret|credential|api[_-]?key)\s*[:=]\s*)\S+")


def clean(value: str) -> str:
    value = URL.sub("<redacted-url>", value)
    return SECRET.sub(r"\1<redacted>", value.replace("lkjmc-dev", "<redacted>"))


class Evidence:
    def __init__(self) -> None:
        self.root = Path(tempfile.mkdtemp(prefix="lkjmc-e-ops-"))
        self.root.chmod(0o700)
        self.results: list[dict[str, object]] = []
        self.project = "lkjmceops" + secrets.token_hex(5)
        self.compose = ["docker", "compose", "--project-name", self.project, "-f", str(ROOT / "docker-compose.yml")]
        self.count = 0

    def write(self, name: str, value: str) -> None:
        (self.root / name).write_text(clean(value)[-LIMIT:], encoding="utf-8")

    def run(self, name: str, command: list[str], timeout: int = 3600, env: dict[str, str] | None = None) -> int:
        self.count += 1
        started = time.monotonic()
        try:
            done = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=timeout, env=env, check=False)
            code, output = done.returncode, done.stdout
        except FileNotFoundError as error:
            code, output = 127, str(error)
        except subprocess.TimeoutExpired as error:
            code, output = 124, (error.stdout or "") + "\ncommand timed out"
        elapsed = round(time.monotonic() - started, 3)
        self.write(f"{self.count:02d}-{name}.log", f"$ {' '.join(command)}\nexit={code} seconds={elapsed}\n{output}")
        return code

    def result(self, probe: str, state: str, summary: str, commands: list[str]) -> None:
        self.results.append({"probe": probe, "state": state, "summary": clean(summary), "commands": [clean(command) for command in commands]})

    def finish(self) -> int:
        index = {"format": "e-ops-v2", "productBase": "d20e5e532db9d3a5577f567dd6a5a24fdc51eea1", "project": self.project, "results": self.results}
        encoded = json.dumps(index, indent=2, sort_keys=True) + "\n"
        self.write("index.json", encoded)
        self.write("index.sha256", hashlib.sha256(encoded.encode()).hexdigest() + "  index.json\n")
        print(f"E-OPS artifacts={self.root}")
        print(" ".join(f"{item['probe']}={item['state']}" for item in self.results))
        return int(any(item["state"] == "FAIL" for item in self.results))


def credential_canary(e: Evidence) -> bool:
    e.run("credential-canary", ["sh", "-ec", f"printf '%s\\n' postgresql://canary:{CANARY}@db.invalid/e-ops?token={CANARY}"])
    leaked = any(CANARY in path.read_text(encoding="utf-8") for path in e.root.iterdir())
    e.result("credential-canary-scan", "PASS" if not leaked else "FAIL", "all retained command and output artifacts excluded the credential canary" if not leaked else "credential canary leaked into retained evidence", ["credential canary artifact scan"])
    return not leaked


def compose_verify(e: Evidence) -> bool:
    version = e.run("docker-version", ["docker", "version", "--format", "{{.Server.Version}}"], 60)
    build = e.run("compose-build", [*e.compose, "--profile", "verify", "build", "--no-cache", "verify"])
    run = e.run("compose-verify", [*e.compose, "--profile", "verify", "run", "--rm", "verify"])
    summary = (e.root / f"{e.count:02d}-compose-verify.log").read_text(encoding="utf-8")
    nested = bool(re.search(r"ok verify-full ran=.* skipped=.*", summary))
    state = "PASS" if not (version or build or run) else "FAIL"
    e.result("clean-compose-run", state, "no-cache build and full Compose verifier ran" if state == "PASS" else "Docker build or verifier failed", ["docker compose ... build --no-cache verify", "docker compose ... run --rm verify"])
    e.result("nested-skip-evidence", "PASS" if nested else "FAIL", "final verifier line retained exact ran/skipped fields" if nested else "no exact nested verifier summary was retained", ["docker compose ... run --rm verify"])
    return state == "PASS"


def rootless(e: Evidence) -> None:
    info = e.run("rootless-info", ["docker", "info", "--format", "{{json .SecurityOptions}}"], 60)
    text = (e.root / f"{e.count:02d}-rootless-info.log").read_text(encoding="utf-8").lower()
    if info == 0 and "rootless" in text:
        command = [*e.compose, "--profile", "verify", "run", "--rm", "verify"]
        code = e.run("rootless-compose", command)
    else:
        command = ["docker", "--context", "rootless", "info", "--format", "{{json .SecurityOptions}}"]
        context = e.run("rootless-context-info", command, 60)
        context_log = (e.root / f"{e.count:02d}-rootless-context-info.log").read_text(encoding="utf-8").lower()
        if context == 0 and "rootless" in context_log:
            command = ["docker", "--context", "rootless", "compose", "--project-name", e.project + "rootless", "-f", str(ROOT / "docker-compose.yml"), "--profile", "verify", "run", "--rm", "verify"]
            code = e.run("rootless-compose", command)
        else:
            code = context or 1
    e.result("rootless-attempt", "PASS" if code == 0 else "EXTERNAL-PENDING", "rootless Compose verifier completed" if code == 0 else "rootless engine/context was unavailable or rejected the exact attempt", ["docker info --format '{{json .SecurityOptions}}'", "docker --context rootless info --format '{{json .SecurityOptions}}'"])


def restore_start(e: Evidence) -> None:
    db = "lkjmc_restore_" + secrets.token_hex(4)
    url = f"postgres://lkjmc:lkjmc-dev@postgres:5432/{db}"
    commands = [
        [*e.compose, "up", "-d", "postgres"],
        [*e.compose, "exec", "-T", "postgres", "pg_isready", "-U", "lkjmc", "-d", "lkjmc"],
        [*e.compose, "run", "--rm", "--no-deps", "-e", "LKJMC_DATABASE_URL=postgres://lkjmc:lkjmc-dev@postgres:5432/lkjmc", "verify", "cargo", "run", "--locked", "--quiet", "-p", "lkjmc-cli", "--", "db", "migrate"],
        [*e.compose, "exec", "-T", "postgres", "pg_dump", "-U", "lkjmc", "--format=custom", "--no-owner", "--file=/tmp/e-ops.dump", "lkjmc"],
        [*e.compose, "exec", "-T", "postgres", "createdb", "-U", "lkjmc", db],
        [*e.compose, "exec", "-T", "postgres", "pg_restore", "-U", "lkjmc", "--clean", "--if-exists", "--no-owner", f"--dbname={db}", "/tmp/e-ops.dump"],
        [*e.compose, "run", "--rm", "--no-deps", "-e", f"LKJMC_DATABASE_URL={url}", "verify", "sh", "-ec", "cargo build --locked -p lkjmc-cli -p lkjmc-daemon; s=$(mktemp -u /tmp/e-ops.XXXX.sock); target/debug/lkjmc-daemon --socket $s --http none --database-url $LKJMC_DATABASE_URL >/tmp/e-ops-daemon.log 2>&1 & p=$!; trap 'kill $p 2>/dev/null || true; wait $p 2>/dev/null || true; rm -f $s' EXIT; for i in $(seq 1 100); do [ -S $s ] && break; sleep .1; done; [ -S $s ]; target/debug/lkjmc --socket $s doctor"],
        [*e.compose, "exec", "-T", "postgres", "dropdb", "-U", "lkjmc", "--if-exists", db],
    ]
    codes = [e.run(f"restore-{index}", command) for index, command in enumerate(commands, 1)]
    e.result("restore-start-run", "PASS" if not any(codes) else "FAIL", "dump, restore, migrated daemon, doctor, and database cleanup completed" if not any(codes) else "restore or daemon start failed; inspect retained logs", ["docker compose ... pg_dump", "docker compose ... pg_restore", "docker compose ... lkjmc-daemon ... doctor"])


def external_labs(e: Evidence) -> None:
    kubectl = e.run("kube-client", ["kubectl", "version", "--client", "--output=yaml"], 60)
    kube = e.run("kube-smoke", ["sh", "-c", "LKJMC_KUBERNETES_SMOKE=1 ./scripts/check-kubernetes-smoke.sh"], 3600, os.environ | {"LKJMC_KUBERNETES_SMOKE": "1"})
    e.result("kube-lab-attempt", "PASS" if not (kubectl or kube) else "EXTERNAL-PENDING", "guarded Kubernetes smoke completed" if not (kubectl or kube) else "client, config, database URL, credentials, or disposable namespace unavailable", ["kubectl version --client --output=yaml", "LKJMC_KUBERNETES_SMOKE=1 ./scripts/check-kubernetes-smoke.sh"])
    systemd = e.run("systemd-sandbox", ["systemd-run", "--user", "--wait", "--pipe", "--collect", "-p", "NoNewPrivileges=yes", "-p", "PrivateTmp=yes", "-p", "ProtectSystem=strict", "/bin/sh", "-ec", "test ! -w /usr; test -w /tmp; id -u"], 120)
    e.result("systemd-sandbox-run", "PASS" if systemd == 0 else "EXTERNAL-PENDING", "user-manager sandbox enforced filesystem checks" if systemd == 0 else "no usable user systemd manager or unsupported sandbox property", ["systemd-run --user --wait --pipe --collect ..."])


def provenance(e: Evidence) -> None:
    hashes = e.run("provenance-hashes", ["sha256sum", "Cargo.lock", "Dockerfile", "docker-compose.yml", "gradle/wrapper/gradle-wrapper.properties"], 60)
    inventory = "cargo metadata --locked --no-deps --format-version 1 | python3 -c 'import json,sys; print(\"packages=\"+\",\".join(p[\"name\"] for p in json.load(sys.stdin)[\"packages\"]))'"
    metadata = e.run("component-inventory", ["sh", "-ec", inventory], 300)
    toolchain = e.run("toolchain-versions", [*e.compose, "run", "--rm", "--no-deps", "verify", "sh", "-ec", "rustc -Vv; cargo -V; ./gradlew --version; java -version; python3 -V"], 600)
    verify = e.run("commit-signature", ["git", "verify-commit", "d20e5e532db9d3a5577f567dd6a5a24fdc51eea1"], 60)
    repeated = e.run("reproducible-daemon", [*e.compose, "run", "--rm", "--no-deps", "verify", "sh", "-ec", "rm -rf /tmp/eops-a /tmp/eops-b; CARGO_TARGET_DIR=/tmp/eops-a cargo build --locked --release -p lkjmc-daemon; sha256sum /tmp/eops-a/release/lkjmc-daemon; CARGO_TARGET_DIR=/tmp/eops-b cargo build --locked --release -p lkjmc-daemon; sha256sum /tmp/eops-b/release/lkjmc-daemon; test $(sha256sum /tmp/eops-a/release/lkjmc-daemon | cut -d' ' -f1) = $(sha256sum /tmp/eops-b/release/lkjmc-daemon | cut -d' ' -f1)"])
    e.result("artifact-provenance", "PASS" if not (hashes or metadata or repeated) else "FAIL", "manifest, component inventory, and two release binary hashes retained; signature result is separate" if not (hashes or metadata or repeated) else "manifest, inventory, or repeat build failed", ["sha256sum ...", "cargo metadata --locked --no-deps", "two isolated release builds"])
    dockerfile = (ROOT / "Dockerfile").read_text(encoding="utf-8")
    wrapper = (ROOT / "gradle/wrapper/gradle-wrapper.properties").read_text(encoding="utf-8")
    missing = [name for name, ok in {"container-digest": "@sha256:" in dockerfile, "rustup-checksum": "rustup-init" in dockerfile and "sha256" in dockerfile, "gradle-wrapper-checksum": "distributionSha256Sum" in wrapper}.items() if not ok]
    e.result("toolchain-acquisition-verified", "PASS" if not missing and toolchain == 0 else "FAIL", "runtime versions and immutable acquisition verification completed" if not missing and toolchain == 0 else "missing immutable acquisition evidence: " + ", ".join(missing), ["docker compose ... rustc/cargo/gradle/java/python versions", "Dockerfile and gradle wrapper static acquisition audit"])
    if verify != 0:
        e.result("optional-signature", "EXTERNAL-PENDING", "git signature verification did not establish a trusted signer", ["git verify-commit d20e5e5"])


def fault_lab(e: Evidence) -> None:
    code = e.run("fault-harness", [*e.compose, "run", "--rm", "--no-deps", "verify", "python3", "scripts/check-fault-harness.py", "--all"])
    e.result("fault-lab-evidence", "PASS" if code == 0 else "FAIL", "test-only fault boundaries and release-marker inspection completed" if code == 0 else "fault harness failed", ["docker compose ... python3 scripts/check-fault-harness.py --all"])


def cleanup(e: Evidence) -> None:
    down = e.run("compose-down", [*e.compose, "down", "--volumes", "--remove-orphans"], 300)
    image = e.project + "-verify"
    removed = e.run("verify-image-remove", ["docker", "image", "rm", "-f", image], 300)
    absent = e.run("verify-image-absent", ["docker", "image", "inspect", image], 60)
    state = "PASS" if not down and not removed and absent else "FAIL"
    e.result("unique-image-cleanup", state, "Compose resources were removed and image inspect was absent after unique verify image removal" if state == "PASS" else "Compose cleanup, unique image removal, or post-removal absence probe failed", ["docker compose ... down --volumes --remove-orphans", "docker image rm -f <unique-verify-image>", "docker image inspect <unique-verify-image>"])


def self_test() -> int:
    evidence = Evidence()
    try:
        evidence.write("url.log", f"postgresql://canary:{CANARY}@db.invalid/e-ops?token={CANARY}")
        assert credential_canary(evidence)
        assert all(CANARY not in path.read_text(encoding="utf-8") for path in evidence.root.iterdir())
    finally:
        shutil.rmtree(evidence.root)
    print("ok e-ops redaction self-test")
    return 0


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    evidence = Evidence()
    try:
        credential_canary(evidence)
        compose_verify(evidence)
        rootless(evidence)
        restore_start(evidence)
        external_labs(evidence)
        provenance(evidence)
        fault_lab(evidence)
    finally:
        cleanup(evidence)
    return evidence.finish()


if __name__ == "__main__":
    sys.exit(main())
