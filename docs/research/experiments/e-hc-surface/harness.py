#!/usr/bin/env python3
"""Loopback-only E-HC-SURFACE research harness; no product imports."""
import argparse, hmac, http.client, importlib.util, json, os, re, secrets, shutil, subprocess, sys, tempfile, threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
SEED = "e-hc-surface-20260712"
PREFIX = "lkjmc-e-hc-surface-"
FIELDS = {"instances": {"id", "state"}, "extension": {"domain", "mode"}}
CORE_QUERY = "{ instances { id state } }"
CORE_RESPONSE = {"data": {"instances": [{"id": "lab-1", "state": "stopped"}]}}


def owned_root(value):
    if value is None:
        root = Path(tempfile.mkdtemp(prefix=PREFIX))
    else:
        root, tmp = Path(value).resolve(), Path(tempfile.gettempdir()).resolve()
        if root.parent != tmp or not root.name.startswith(PREFIX):
            raise ValueError("artifact root must be an owned /tmp/lkjmc-e-hc-surface-* path")
        root.mkdir(mode=0o700, exist_ok=False)
    (root / ".owned").write_text("E-HC-SURFACE\n", encoding="utf-8")
    return root

def mobile_evidence(path):
    try:
        data = json.loads(Path(path).read_text(encoding="utf-8"))
    except FileNotFoundError:
        return None, "missing-evidence"
    except json.JSONDecodeError:
        return None, "invalid-json"
    capability, evidence_id = (data.get("uniqueCapability"), data.get("evidenceId")) if isinstance(data, dict) else (None, None)
    if not isinstance(evidence_id, str) or not isinstance(capability, str):
        return None, "missing-unique-fields"
    if capability.strip().lower() in {"instance status", "graph projection"}:
        return None, "duplicated-by-graph"
    return {"evidenceId": evidence_id, "uniqueCapability": capability}, None

def load_domain(root):
    source = root / "third_party_status.py"
    source.write_text('DOMAIN = "third.party.status"\n\ndef projection():\n    return {"domain": DOMAIN, "mode": "read-only"}\n', encoding="utf-8")
    try:
        spec = importlib.util.spec_from_file_location("third_party_status", source)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        domain = module.projection()
        if domain != {"domain": "third.party.status", "mode": "read-only"}:
            raise ValueError("third-party domain shape denied")
        return domain
    finally:
        sys.modules.pop("third_party_status", None)
        source.unlink(missing_ok=True)


class SurfaceHandler(BaseHTTPRequestHandler):
    state = None

    def log_message(self, *_args): return

    def send(self, status, body, content_type="application/json"):
        port = self.server.server_port
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Cache-Control", "no-store")
        self.send_header("Content-Security-Policy", f"default-src 'none'; script-src 'self'; connect-src http://127.0.0.1:{port}; base-uri 'none'; form-action 'none'")
        self.send_header("X-Frame-Options", "DENY")
        self.send_header("Referrer-Policy", "no-referrer")
        self.end_headers(); self.wfile.write(body)

    def reply(self, status, data): self.send(status, json.dumps(data, sort_keys=True).encode("utf-8"))

    def do_GET(self):
        if self.path == "/operator-task":
            return self.send(200, b'<!doctype html><main id="status">loading</main><script src="/operator-task.js"></script>', "text/html; charset=utf-8")
        if self.path == "/operator-task.js":
            task = b'''const status=document.getElementById("status");fetch("/graph",{method:"POST",headers:{"Content-Type":"application/json"},body:'{"query":"{ instances { id state } }"}'}).then(async r=>{const x=await r.json();if(!r.ok||x.data.instances[0].state!=="stopped")throw Error("graph response denied");status.textContent=x.data.instances[0].state;return fetch("/browser-evidence",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({status:r.status,response:x})})}).then(r=>{if(!r.ok)throw Error("evidence denied")}).catch(()=>status.textContent="denied");'''
            return self.send(200, task, "text/javascript; charset=utf-8")
        if self.path == "/mobile/v1/status" and self.state.mobile:
            return self.reply(200, {"data": {"id": "lab-1", "state": "stopped"}})
        self.reply(404, {"error": "not found"})

    def do_POST(self):
        if self.path == "/graph": return self.graph()
        if self.path == "/browser-evidence": return self.browser_evidence()
        if self.path == "/public/control": return self.public_control()
        self.reply(404, {"error": "not found"})

    def json_body(self):
        try: return json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))))
        except (json.JSONDecodeError, ValueError): return None

    def graph(self):
        data = self.json_body(); query = data.get("query", "") if isinstance(data, dict) else ""
        match = re.fullmatch(r"\{\s*(instances|extension)\s*\{\s*([a-z ]+?)\s*\}\s*\}", query)
        if not match: return self.reply(400, {"error": "query denied"})
        root, requested = match.groups(); fields = requested.split()
        if not fields or len(fields) != len(set(fields)) or not set(fields) <= FIELDS[root]:
            return self.reply(400, {"error": "selection denied"})
        record = CORE_RESPONSE["data"]["instances"][0] if root == "instances" else self.state.domain
        value = [{key: record[key] for key in fields}] if root == "instances" else {key: record[key] for key in fields}
        response = {"data": {root: value}}
        self.state.graph_posts.append({"query": query, "response": response})
        self.reply(200, response)

    def browser_evidence(self):
        agent, data = self.headers.get("User-Agent", ""), self.json_body()
        name = "firefox" if "Firefox/" in agent else "chrome" if "Chrome/" in agent else None
        if name is None or not isinstance(data, dict) or data != {"status": 200, "response": CORE_RESPONSE}:
            return self.reply(400, {"error": "browser evidence denied"})
        self.state.browser_evidence.append(name)
        self.reply(200, {"result": "recorded"})

    def public_control(self):
        if int(self.headers.get("Content-Length", "0")) > 1024: return self.reply(413, {"error": "body denied"})
        if self.headers.get("Origin"): return self.reply(403, {"error": "origin denied"})
        if not hmac.compare_digest(self.headers.get("Authorization", ""), "Bearer " + self.state.token):
            return self.reply(403, {"error": "credential denied"})
        self.rfile.read(int(self.headers.get("Content-Length", "0")))
        self.reply(405, {"error": "public control disabled"})


def request(port, method, path, body=b"", headers=None):
    connection = http.client.HTTPConnection("127.0.0.1", port, timeout=3)
    connection.request(method, path, body=body, headers=headers or {})
    response = connection.getresponse(); result = response.status, response.read(), dict(response.getheaders())
    connection.close(); return result

def browser_attempts(url, root, state):
    attempts = {}
    for name, binary in (("chrome", shutil.which("google-chrome")), ("firefox", shutil.which("firefox"))):
        if not binary:
            attempts[name] = {"status": "BLOCKED", "detail": "unavailable"}; continue
        posts, evidence = len(state.graph_posts), state.browser_evidence.count(name)
        profile = root / (name + "-profile"); profile.mkdir()
        command = [binary, "--headless", "--disable-gpu", f"--user-data-dir={profile}", "--dump-dom", url] if name == "chrome" else [binary, "--headless", "--profile", str(profile), "--screenshot", str(root / "browser.png"), url]
        try: subprocess.run(command, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=15, check=False)
        except subprocess.TimeoutExpired: pass
        shutil.rmtree(profile, ignore_errors=True); (root / "browser.png").unlink(missing_ok=True)
        posted = any(item == {"query": CORE_QUERY, "response": CORE_RESPONSE} for item in state.graph_posts[posts:])
        passed = posted and state.browser_evidence.count(name) == evidence + 1
        attempts[name] = {"status": "PASS" if passed else "BLOCKED", "detail": "POST /graph and response validated" if passed else "no semantic loopback request"}
    return attempts

def browser_aggregate(attempts):
    statuses = {attempt["status"] for attempt in attempts.values()}
    return "PASS" if statuses == {"PASS"} else "BLOCKED" if statuses == {"BLOCKED"} else "MIXED"

def scan_artifacts(root, token):
    items, files = list(root.rglob("*")), [item for item in root.rglob("*") if item.is_file()]
    if any(item.name in {"__pycache__", "third_party_status.py"} for item in items) or any(token.encode() in item.read_bytes() for item in files):
        raise AssertionError("secret or temporary import artifact reached root")
    return len(files)


def exercise(root, mobile):
    SurfaceHandler.state = type("State", (), {"token": secrets.token_urlsafe(24), "domain": load_domain(root), "mobile": mobile, "graph_posts": [], "browser_evidence": []})()
    server = ThreadingHTTPServer(("127.0.0.1", 0), SurfaceHandler); thread = threading.Thread(target=server.serve_forever, daemon=True); thread.start(); cases = {}
    try:
        def check(name, expected, *args, **kwargs):
            status, body, headers = request(server.server_port, *args, **kwargs)
            if status != expected: raise AssertionError(f"{name} expected {expected} got {status}")
            cases[name] = status; return body, headers
        _, headers = check("operator-task", 200, "GET", "/operator-task")
        expected_csp = f"connect-src http://127.0.0.1:{server.server_port}"
        if expected_csp not in headers.get("Content-Security-Policy", ""): raise AssertionError("loopback CSP missing")
        check("operator-task-script", 200, "GET", "/operator-task.js")
        graph, _ = check("graph-core", 200, "POST", "/graph", json.dumps({"query": CORE_QUERY}).encode())
        if json.loads(graph) != CORE_RESPONSE: raise AssertionError("core projection missing")
        extension, _ = check("graph-loaded-domain", 200, "POST", "/graph", b'{"query":"{ extension { domain mode } }"}')
        if b"third.party.status" not in extension: raise AssertionError("loaded domain missing")
        check("graph-introspection", 400, "POST", "/graph", b'{"query":"{ __schema { types } }"}')
        check("graph-mutation", 400, "POST", "/graph", b'{"query":"mutation { instances { id } }"}')
        check("public-no-bearer", 403, "POST", "/public/control", b'{"action":"start"}')
        check("public-wrong-bearer", 403, "POST", "/public/control", b"{}", {"Authorization": "Bearer wrong"})
        check("public-foreign-origin", 403, "POST", "/public/control", b"{}", {"Authorization": "Bearer " + SurfaceHandler.state.token, "Origin": "https://evil.invalid"})
        check("public-valid-bearer-disabled", 405, "POST", "/public/control", b"{}", {"Authorization": "Bearer " + SurfaceHandler.state.token})
        check("public-oversize", 413, "POST", "/public/control", b"x" * 1025); check("mobile-unregistered", 404, "GET", "/mobile/v1/status")
        attempts = browser_attempts(f"http://127.0.0.1:{server.server_port}/operator-task", root, SurfaceHandler.state)
    finally:
        server.shutdown(); server.server_close(); thread.join(timeout=3)
    retained = scan_artifacts(root, SurfaceHandler.state.token) + 1
    summary = {"seed": SEED, "listener": "127.0.0.1:ephemeral", "cases": cases, "domain": SurfaceHandler.state.domain, "browserAttempts": attempts, "browserStatus": browser_aggregate(attempts), "mobile": "registered" if mobile else "not-registered", "mutations": 0, "secretScan": "pass", "scannedFiles": retained}
    (root / "summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    if scan_artifacts(root, SurfaceHandler.state.token) != retained: raise AssertionError("artifact scan changed")
    return summary


def main():
    parser = argparse.ArgumentParser(); parser.add_argument("--self-test", action="store_true"); parser.add_argument("--browser-semantic-test", action="store_true"); parser.add_argument("--artifact-root"); parser.add_argument("--mobile-evidence"); args = parser.parse_args()
    if not (args.self_test or args.browser_semantic_test): parser.error("--self-test or --browser-semantic-test is required")
    mobile = None
    if args.mobile_evidence:
        mobile, blocked = mobile_evidence(args.mobile_evidence)
        if blocked: print(f"E-HC-SURFACE mobile=BLOCKED reason={blocked}"); return 2
    root = owned_root(args.artifact_root); summary = exercise(root, mobile)
    print(f"E-HC-SURFACE result=PASS cases={len(summary['cases'])} browser={summary['browserStatus']} mobile={summary['mobile']} artifact={root}")
    return 0


if __name__ == "__main__":
    sys.dont_write_bytecode = True; os.umask(0o077); raise SystemExit(main())
