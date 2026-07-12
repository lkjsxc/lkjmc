#!/usr/bin/env python3
"""Run the bounded E-NETWORK compiler comparison outside product code."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
DOCUMENT = Path(__file__).resolve().parent.parent / "experiments/e-network-document.json"
def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"))
def digest(value):
    return hashlib.sha256(canonical(value).encode()).hexdigest()
def compile_document(document, runtime):
    policy, nodes = document["runtimePolicy"], document["nodes"]
    if document.get("version") != 1 or runtime not in policy["allowed"]:
        raise ValueError("unsupported runtime policy")
    by_id = {node["id"]: node for node in nodes}
    if len(by_id) != len(nodes) or {"proxy", "hub"} - {n["role"] for n in nodes}:
        raise ValueError("proxy and hub are required exactly once")
    routes = [document["routing"]["default"], *document["routing"]["optional"].values()]
    if any(target not in by_id for target in routes):
        raise ValueError("routing names an absent node")
    selected = [node for node in nodes if node["enabled"]]
    if document["routing"]["default"] not in {node["id"] for node in selected}:
        raise ValueError("default route is not enabled")
    for node in selected:
        if node["asset"] not in document["assets"] or any(p not in document["plugins"] for p in node["plugins"]):
            raise ValueError("asset or plugin is not declared")
    resources = [{"id": node["id"], "role": node["role"], "spec": digest({
        "node": node, "asset": document["assets"][node["asset"]],
        "plugins": {p: document["plugins"][p] for p in node["plugins"]},
        "routes": document["routing"] if node["role"] == "proxy" else {}, "policy": policy,
    })} for node in selected]
    result = {"runtime": runtime, "resources": resources, "document": digest(document)}
    if runtime == "kubernetes":
        result["manifests"] = manifests(document, resources)
    return result

def manifests(document, resources):
    objects = []
    for resource in resources:
        name = "research-" + resource["id"]
        labels = {"experiment": "e-network", "node": resource["id"]}
        encoded = canonical({"document": document, "resource": resource})
        objects.extend([
            {"kind": "ConfigMap", "metadata": {"name": name, "labels": labels}, "data": {"network.json": encoded}},
            {"kind": "Deployment", "metadata": {"name": name, "labels": labels}, "spec": {"replicas": 1, "template": {"spec": {"containers": [{"name": resource["role"], "image": "research/" + resource["role"] + "@" + resource["spec"]}]}}}},
            {"kind": "Service", "metadata": {"name": name, "labels": labels}, "spec": {"selector": labels}},
        ])
    return objects

class LocalAdapter:
    def __init__(self, root):
        self.root, self.live, self.events = root, {}, []
        root.mkdir()
    def start(self, resource):
        process = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"], cwd=self.root)
        if process.poll() is not None:
            raise RuntimeError("local child exited before observation")
        self.live[resource["id"]] = (process, resource["spec"])
        self.events.append({"effect": "start", "id": resource["id"], "pid": process.pid})
    def stop(self, node_id):
        process, _ = self.live.pop(node_id)
        process.terminate()
        try:
            process.wait(timeout=3)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=3)
        self.events.append({"effect": "stop", "id": node_id, "pid": process.pid, "exit": process.returncode})
    def inspect(self):
        return {key: value[1] for key, value in self.live.items() if value[0].poll() is None}
    def apply(self, plan, fail_after=None):
        desired, before = {item["id"]: item for item in plan["resources"]}, self.inspect()
        replace = [node_id for node_id, item in desired.items() if node_id in before and before[node_id] != item["spec"]]
        started = []
        try:
            for node_id in replace:
                self.stop(node_id)
            for node_id, item in desired.items():
                if node_id not in before or node_id in replace:
                    self.start(item); started.append(node_id)
                    if node_id == fail_after: raise RuntimeError("controlled post-start effect failure")
            removed = sorted((set(before) - set(desired)) | set(replace))
            for node_id in set(before) - set(desired): self.stop(node_id)
            return {"started": started, "stopped": removed, "observed": self.inspect()}
        except Exception:
            for node_id in reversed(started):
                if node_id in self.live: self.stop(node_id)
            raise
    def close(self):
        for node_id in list(self.live):
            self.stop(node_id)

def changed(document):
    value = copy.deepcopy(document)
    next(node for node in value["nodes"] if node["id"] == "events")["enabled"] = True
    return value

def removed(document):
    value = changed(document)
    value["nodes"] = [node for node in value["nodes"] if node["id"] != "events"]
    value["routing"]["optional"].pop("events")
    return value

def local_case(root, label, work):
    adapter = LocalAdapter(root / label)
    try:
        passed, observed = work(adapter), adapter.inspect()
    finally:
        adapter.close()
    (adapter.root / "events.json").write_text(json.dumps(adapter.events, indent=2) + "\n")
    return {"passed": passed, "observed": observed, "events": adapter.events}

def imperative(plan, root):
    children = []
    try:
        for _ in range(2):
            for _resource in plan["resources"]:
                children.append(subprocess.Popen([sys.executable, "-c", "import time; time.sleep(60)"], cwd=root))
        live = sum(child.poll() is None for child in children)
        return {"firstApplyStarts": len(plan["resources"]), "secondApplyStarts": len(plan["resources"]), "liveChildren": live}
    finally:
        for child in children:
            if child.poll() is None:
                child.terminate()
                child.wait(timeout=3)

def kube_attempt():
    command = ["kubectl", "version", "--client", "--output=yaml"]
    try:
        done = subprocess.run(command, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=20, check=False)
        code, output = done.returncode, done.stdout[-4096:]
    except FileNotFoundError as error:
        code, output = 127, str(error)
    except subprocess.TimeoutExpired as error:
        code, output = 124, str(error)
    configured = os.environ.get("LKJMC_KUBERNETES_SMOKE") == "1" and bool(os.environ.get("LKJMC_KUBERNETES_NAMESPACE"))
    return {"command": " ".join(command), "exit": code, "output": output, "outcome": "PASS" if code == 0 and configured else "BLOCKED", "reason": "cluster apply not attempted; require client success plus LKJMC_KUBERNETES_SMOKE=1 and LKJMC_KUBERNETES_NAMESPACE"}

def run(raw):
    base = json.loads(DOCUMENT.read_text(encoding="utf-8"))
    baseline, update, deletion = compile_document(base, "local-process"), compile_document(changed(base), "local-process"), compile_document(removed(base), "local-process")
    results = {}
    results["inspect-apply"] = local_case(raw, "inspect", lambda adapter: (adapter.inspect() == {} and adapter.apply(baseline)["observed"] == {item["id"]: item["spec"] for item in baseline["resources"]}))
    results["reapply"] = local_case(raw, "reapply", lambda adapter: (adapter.apply(baseline), adapter.apply(baseline))[1]["started"] == [])
    def changed_case(adapter):
        adapter.apply(baseline); return adapter.apply(update)["started"] == ["events"]
    results["change"] = local_case(raw, "change", changed_case)
    def rollback_case(adapter):
        adapter.apply(baseline); adapter.apply(update); return adapter.apply(baseline)["stopped"] == ["events"]
    results["rollback"] = local_case(raw, "rollback", rollback_case)
    def removal_case(adapter):
        adapter.apply(update); return adapter.apply(deletion)["stopped"] == ["events", "proxy"]
    results["removal"] = local_case(raw, "removal", removal_case)
    def failure_case(adapter):
        adapter.apply(baseline)
        try: adapter.apply(update, fail_after="events")
        except RuntimeError: pass
        return "events" not in adapter.inspect() and [event["effect"] for event in adapter.events].count("start") == 3 and [event["effect"] for event in adapter.events].count("stop") >= 1
    results["failure"] = local_case(raw, "failure", failure_case)
    try: compile_document({**base, "runtimePolicy": {"allowed": ["local-process"], "default": "local-process"}}, "kubernetes"); unsupported = False
    except ValueError: unsupported = True
    results["unsupported"] = {"passed": unsupported, "observed": "pure compiler rejected kubernetes"}
    kube = compile_document(base, "kubernetes")
    manifests_ok = len(kube["manifests"]) == 6 and {item["kind"] for item in kube["manifests"]} == {"ConfigMap", "Deployment", "Service"}
    results["kubernetes-manifests"] = {"passed": manifests_ok, "observed": len(kube["manifests"])}
    imperative_result = imperative(baseline, raw)
    results["imperative-bootstrap"] = {"passed": imperative_result["liveChildren"] == 4, "observed": imperative_result}
    result = {"experiment": "E-NETWORK", "document": base, "plans": {"local": baseline, "kubernetes": kube}, "probes": results, "kubernetesCapability": kube_attempt(), "passed": all(value["passed"] for value in results.values())}
    (raw / "summary.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    files = {path.relative_to(raw).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest() for path in raw.rglob("*") if path.is_file() and path.name != "index.json"}
    (raw / "index.json").write_text(json.dumps({"files": files}, indent=2) + "\n")
    print(f"E-NETWORK run={'PASS' if result['passed'] else 'FAIL'} raw={raw}")
    print(f"E-NETWORK kubernetes={result['kubernetesCapability']['outcome']} exit={result['kubernetesCapability']['exit']}")
    return 0 if result["passed"] else 1

def replay(raw):
    try:
        if raw.parent != Path(tempfile.gettempdir()).resolve() or not raw.name.startswith("lkjmc-e-network-"): raise ValueError()
        files = json.loads((raw / "index.json").read_text())["files"]
        valid = all((raw / name).is_file() and hashlib.sha256((raw / name).read_bytes()).hexdigest() == value for name, value in files.items())
    except (OSError, ValueError, KeyError, json.JSONDecodeError): valid = False
    print("E-NETWORK replay=" + ("PASS" if valid else "BLOCKED")); return 0 if valid else 2

def main():
    parser = argparse.ArgumentParser(); parser.add_argument("action", choices=("run", "replay", "cleanup")); parser.add_argument("--raw-dir", type=Path); args = parser.parse_args()
    if args.action == "run": return run(Path(tempfile.mkdtemp(prefix="lkjmc-e-network-")))
    if not args.raw_dir: return 2
    if args.action == "replay": return replay(args.raw_dir.resolve())
    if args.raw_dir.parent == Path(tempfile.gettempdir()).resolve() and args.raw_dir.name.startswith("lkjmc-e-network-"): shutil.rmtree(args.raw_dir); return 0
    return 2

if __name__ == "__main__":
    raise SystemExit(main())
