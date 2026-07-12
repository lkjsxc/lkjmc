#!/usr/bin/env python3
"""Capture bounded E-MENU evidence without enabling a Java daemon menu."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import time
import zipfile
ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "docs/research/experiments/e-menu-20260712-source-manifest.json"
PREFIX, CAP = "lkjmc-e-menu-", 8192

def digest(value): return hashlib.sha256(value).hexdigest()
def redact(text):
    return re.sub(r"(?i)(token|password|secret)\s*[=:]\s*\S+", r"\1=<redacted>", text)[:CAP]
def write(raw, name, record):
    output = record.pop("output", "")
    (raw / f"{name}.txt").write_text(output, encoding="utf-8")
    (raw / f"{name}.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
def execute(name, args, raw, env=None, mode="pass", marker=""):
    values = os.environ.copy(); values.update(env or {})
    started = time.monotonic()
    try:
        done = subprocess.run(args, cwd=ROOT, env=values, text=True, stdout=subprocess.PIPE,
                              stderr=subprocess.STDOUT, check=False, timeout=1800)
        code, output = done.returncode, redact(done.stdout)
    except (OSError, subprocess.TimeoutExpired) as error: code, output = 124, redact(str(error))
    observed = marker in output if marker else True
    outcome = ("BLOCKED" if code and observed else "FAIL") if mode == "blocked" else (
        "OBSERVED_BOUNDARY" if code and observed else "FAIL") if mode == "boundary" else (
        "PASS" if not code and observed else "BLOCKED") if mode == "optional" else (
        "PASS" if not code and observed else "FAIL")
    record = {"name": name, "command": " ".join(args), "exit": code,
              "elapsedSeconds": round(time.monotonic() - started, 3), "outcome": outcome,
              "output": output}
    write(raw, name, record); return record
def load_manifest():
    value = json.loads(MANIFEST.read_text(encoding="utf-8"))
    required = {"format", "rootFiles", "docsGlob", "excludedPrefixes", "excludedPaths"}
    if value.get("format") != "e-menu-compiled-source-manifest-v1" or not required <= value.keys():
        raise ValueError("invalid E-MENU source manifest")
    return value
def source_paths(manifest):
    excluded = set(manifest["excludedPaths"])
    paths = list(manifest["rootFiles"])
    for path in sorted(ROOT.glob(manifest["docsGlob"])):
        relative = path.relative_to(ROOT).as_posix()
        if not relative.startswith(tuple(manifest["excludedPrefixes"])) and relative not in excluded:
            paths.append(relative)
    if len(paths) != len(set(paths)) or any(not (ROOT / path).is_file() for path in paths):
        raise ValueError("invalid manifest source paths")
    return sorted(paths)
def provenance(manifest, paths):
    files = {path: digest((ROOT / path).read_bytes()) for path in paths}
    canonical = json.dumps(files, sort_keys=True, separators=(",", ":")).encode()
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    return {"repositoryHead": head, "manifestSha256": digest(MANIFEST.read_bytes()),
            "manifestSourceTip": digest(canonical), "files": files}
def width(value): return sum(2 if ord(char) > 0x2E80 else 1 for char in value)
def wrap(lines, limit=36):
    result = []
    for line in lines:
        remaining = line.rstrip()
        while remaining and width(remaining) > limit:
            used = cut = 0; space = -1
            for index, char in enumerate(remaining):
                if char.isspace(): space = index
                if used + (2 if ord(char) > 0x2E80 else 1) > limit: cut = space if space > 0 else index; break
                used += 2 if ord(char) > 0x2E80 else 1
            result.append(remaining[:cut].rstrip()); remaining = remaining[cut:].lstrip()
        if remaining: result.append(remaining)
    return result
def normalized(path):
    if not path or not path.strip(): return ""
    value = path.replace("\\", "/").split("#", 1)[0]
    if value.startswith("/") or ":" in value.split("/", 1)[0]: return None
    stack = []
    for part in value.split("/"):
        if not part or part == ".": continue
        if part == "..":
            if not stack: return None
            stack.pop()
        else: stack.append(part)
    return "/".join(stack)
def compiled_bundle(raw, paths, proof):
    jars = sorted((ROOT / "platforms/jvm/paper/build/libs").glob("*-all.jar"))
    if not jars: return {"name": "compiled-resource", "outcome": "FAIL", "reason": "paper shadow jar missing"}
    with zipfile.ZipFile(jars[-1]) as jar: payload = jar.read("lkjmc-docs-bundle.json")
    bundle = json.loads(payload); files = {item["path"]: item for item in bundle["files"]}
    selected = sorted(set(paths)); extras = sorted(set(files) - set(selected))
    if set(selected) - set(files) or extras != ["docs/research/decisions/e-menu-20260712.md", "docs/research/runs/e-menu-20260712.md"]:
        return {"name": "compiled-resource", "outcome": "FAIL", "reason": "manifest/resource mismatch"}
    chosen = [files[path] for path in selected]
    selected_payload = json.dumps({"version": bundle["version"], "files": chosen}, ensure_ascii=False,
                                  separators=(",", ":")).encode()
    root = [{"slot": index, "material": "BOOK", "title": item["title"], "action": "file:" + item["path"]}
            for index, item in enumerate(chosen[:45])]
    root.append({"slot": 53, "material": "BARRIER", "title": "Close", "action": "close"})
    page = wrap(files["docs/README.md"]["lines"])[:10]
    detail = [{"slot": 19 + index, "material": "PAPER", "title": line} for index, line in enumerate(page)]
    detail += [{"slot": 49, "material": "BOOK", "title": "Documentation", "action": "root"},
               {"slot": 53, "material": "BARRIER", "title": "Close", "action": "close"}]
    layout = {"kind": "renderer-derived manifest-selected compiled-resource layout specimen", "notPlayerProof": True,
              "manifestSourceTip": proof["manifestSourceTip"], "resourceSha256": digest(selected_payload),
              "source": "platforms/jvm/paper/src/main/java/com/lkjmc/paper/LocalDocsMenu.java",
              "root": root, "docsReadmePageZero": detail}
    (raw / "compiled-local-docs-layout.json").write_text(json.dumps(layout, indent=2) + "\n", encoding="utf-8")
    route = {"normalized": {value: normalized(value) for value in ("docs//./README.md", "../AGENTS.md", "/etc/passwd", "docs/README.md#purpose")}, "missingPathCandidate": "local-search", "candidateOnly": True}
    (raw / "route-failure-candidate.json").write_text(json.dumps(route, indent=2) + "\n", encoding="utf-8")
    return {"name": "compiled-resource", "outcome": "PASS", "jar": jars[-1].name,
            "allBundleDocuments": len(files), "manifestDocuments": len(chosen), "excludedBundlePaths": extras,
            "allResourceSha256": digest(payload), "manifestResourceSha256": digest(selected_payload),
            "manifestSourceTip": proof["manifestSourceTip"], "layoutKind": layout["kind"]}
def compose_image_provenance(raw):
    names = subprocess.run(["docker", "compose", "--profile", "verify", "config", "--images"], cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False)
    images = [line for line in names.stdout.splitlines() if line.strip() and line != "postgres:16-alpine"]
    if names.returncode or len(images) != 1:
        record = {"name": "compose-image-provenance", "exit": names.returncode, "outcome": "FAIL", "output": redact(names.stdout)}
        write(raw, "compose-image-provenance", record); return record
    return execute("compose-image-provenance", ["docker", "image", "inspect", "--format", "{{json .}}", images[0]], raw)
def malformed_probe(raw, jar):
    source = raw / "MalformedBundleProbe.java"
    source.write_text('import com.lkjmc.common.docs.DocBundle; import java.io.ByteArrayInputStream; public final class MalformedBundleProbe { public static void main(String[] a) { try { DocBundle.load(new ByteArrayInputStream("{".getBytes())); System.exit(1); } catch (RuntimeException e) { System.out.println(e.getClass().getName()); } } }\n')
    built = subprocess.run(["javac", "-cp", str(jar), "-d", str(raw), str(source)], cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, check=False)
    if built.returncode: return {"name": "malformed-compiled-loader", "exit": built.returncode, "outcome": "FAIL", "output": redact(built.stdout)}
    return execute("malformed-compiled-loader", ["java", "-cp", f"{raw}:{jar}", "MalformedBundleProbe"], raw, marker="Json")
def outage_probe(raw):
    build, runtime = Path(tempfile.mkdtemp(prefix="lkjmc-e-menu-build-")), Path(tempfile.mkdtemp(prefix="lkjmc-e-menu-runtime-"))
    env = os.environ.copy(); env.pop("LKJMC_DATABASE_URL", None); env["CARGO_TARGET_DIR"] = str(build)
    try:
        if execute("daemon-outage-build", ["cargo", "build", "--locked", "-p", "lkjmc-cli", "-p", "lkjmc-daemon"], raw, env)["outcome"] != "PASS": return {"outcome": "FAIL"}
        socket = runtime / "daemon.sock"; daemon = subprocess.Popen([str(build / "debug/lkjmc-daemon"), "--socket", str(socket), "--http", "none"], cwd=ROOT, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            for _ in range(100):
                if socket.exists(): return execute("daemon-view-model-outage", [str(build / "debug/lkjmc"), "--json", "--socket", str(socket), "instance", "list"], raw, env, "boundary", "database")
                time.sleep(.05)
            return {"name": "daemon-view-model-outage", "outcome": "FAIL"}
        finally: daemon.terminate(); daemon.wait(timeout=10)
    finally: shutil.rmtree(build, ignore_errors=True); shutil.rmtree(runtime, ignore_errors=True)
def capture(raw_dir=None):
    raw = raw_dir.resolve() if raw_dir else Path(tempfile.mkdtemp(prefix=PREFIX))
    if raw_dir and (raw.parent != Path(tempfile.gettempdir()).resolve() or not raw.name.startswith(PREFIX) or raw.exists()):
        print("E-MENU capture=BLOCKED unsafe-raw-dir"); return 2
    if raw_dir: raw.mkdir()
    manifest = load_manifest(); paths = source_paths(manifest); proof = provenance(manifest, paths)
    (raw / "source-provenance.json").write_text(json.dumps(proof, indent=2) + "\n", encoding="utf-8")
    results = [execute("static-catalog", ["./scripts/check-menus.py"], raw), execute("generated-catalog", ["./scripts/generate-menu-docs.py", "--check"], raw), execute("locale", ["./scripts/check-locales.py"], raw), execute("route-core", ["./gradlew", "--no-daemon", "--no-build-cache", ":platforms:jvm:common:test", "--tests", "com.lkjmc.common.docs.DocCoreTest"], raw), execute("local-docs-surface", ["./gradlew", "--no-daemon", "--no-build-cache", ":platforms:jvm:paper:test", "--tests", "com.lkjmc.paper.LocalPaperSurfaceTest"], raw), execute("compiled-paper-bundle", ["./gradlew", "--no-daemon", "--no-build-cache", ":platforms:jvm:paper:shadowJar"], raw)]
    bundle = compiled_bundle(raw, paths, proof); write(raw, "compiled-resource", bundle); results.append(bundle)
    if bundle["outcome"] == "PASS": results.append(malformed_probe(raw, ROOT / "platforms/jvm/paper/build/libs" / bundle["jar"]))
    results += [outage_probe(raw), execute("daemon-view-model-candidates", ["docker", "compose", "--profile", "verify", "run", "--rm", "--build", "verify", "cargo", "test", "--locked", "-p", "lkjmc-daemon", "menu_data_commands_return_documented_shapes_when_database_configured"], raw), compose_image_provenance(raw), execute("java-protocol-player", ["./scripts/check-minecraft-smoke.sh"], raw, {"LKJMC_MINECRAFT_SMOKE": "1"}, "blocked", "blocked:"), execute("playable-protocol-player", ["./scripts/check-playable-smoke.sh"], raw, {"LKJMC_PLAYABLE_SMOKE": "1", "LKJMC_ACCEPT_MINECRAFT_EULA": "1"}, "blocked", "blocked:")]
    summary = {"experiment": "E-MENU", "seed": 20260712, "sourceProvenance": {key: proof[key] for key in ("repositoryHead", "manifestSha256", "manifestSourceTip")}, "results": [{key: value for key, value in result.items() if key != "output"} for result in results], "limits": ["Layout specimens derive from manifest-selected compiled resources and source slots, not Bukkit.", "Daemon response candidates have no Java consumer.", "Java/protocol player routes remain BLOCKED by F-SAFETY."]}
    (raw / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    index = {path.name: digest(path.read_bytes()) for path in sorted(raw.iterdir()) if path.name != "index.json"}; (raw / "index.json").write_text(json.dumps({"files": index}, indent=2) + "\n", encoding="utf-8")
    failed = [result.get("name", "unknown") for result in results if result.get("outcome") == "FAIL"]
    print(f"E-MENU capture={'FAIL' if failed else 'PASS'} raw={raw}"); print(f"replay=(cd /tmp && python3 {Path(__file__).resolve()} replay --raw-dir {raw})")
    return 1 if failed else 0
def replay(raw):
    root = raw.resolve()
    try:
        index = json.loads((root / "index.json").read_text())["files"]; proof = json.loads((root / "source-provenance.json").read_text())
        valid = root.parent == Path(tempfile.gettempdir()).resolve() and root.name.startswith(PREFIX) and all((root / name).is_file() and digest((root / name).read_bytes()) == value for name, value in index.items())
        manifest = load_manifest(); current = provenance(manifest, source_paths(manifest))
        valid = valid and all(proof[key] == current[key] for key in ("manifestSha256", "manifestSourceTip", "files"))
        valid = valid and subprocess.run(["git", "rev-parse", "--verify", proof["repositoryHead"] + "^{commit}"], cwd=ROOT, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False).returncode == 0
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError): valid = False
    print("E-MENU replay=" + ("PASS" if valid else "BLOCKED")); return 0 if valid else 2
def main():
    parser = argparse.ArgumentParser(); commands = parser.add_subparsers(dest="action", required=True); capture_parser = commands.add_parser("capture"); capture_parser.add_argument("--raw-dir", type=Path); replay_parser = commands.add_parser("replay"); replay_parser.add_argument("--raw-dir", type=Path, required=True); args = parser.parse_args(); return capture(args.raw_dir) if args.action == "capture" else replay(args.raw_dir)
if __name__ == "__main__": raise SystemExit(main())
