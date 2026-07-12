"""Negative truth-probe fixtures for contract and repository boundaries."""
import json
import tempfile
from pathlib import Path


def write(root, path, value):
    target = root / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(value, encoding="utf-8")


def fixture(root, reopened, forensic, goldens, boundaries):
    forensic_items = [{"priorTask": item, "classification": "reopened"} for item in sorted(reopened)]
    mapped = [{"priorItem": item, "futureTask": "A-NEXT", "probe": "probe-mutation-tests"} for item in sorted(reopened)]
    write(root, forensic, json.dumps({"items": forensic_items}))
    write(root, "contracts/truth-probe-mapping.json", json.dumps({"items": mapped}))
    write(root, "contracts/commands/README.json", '{"format":"lkjmc-command-shards-v1","shards":["asset-01.json"]}')
    write(root, "contracts/commands/asset-01.json", asset_contract())
    write(root, "crates/lkjmc-core/src/command_shards.rs", 'include_str!("../../../contracts/commands/asset-01.json");\n')
    write(root, "crates/lkjmc-cli/src/client.rs", 'command_registry::validate_body(command, &body)\ndaemon_command("asset.plugin.sync", json!({"plugin":"paper"}))\n')
    write(root, "crates/lkjmc-daemon/src/web/api.rs", "crate::dispatch::dispatch_as(\n")
    write(root, "scripts/check-docs.py", "def scan():\n    return ROOT.rglob('*.md')\n")
    write(root, "docs/owner.md", "`scripts/real.py`\n")
    write(root, "scripts/real.py", "pass\n")
    write(root, "crates/lkjmc-daemon/src/app.rs", "pub runtime: RuntimeCoordinator;\n")
    for path, call in boundaries:
        write(root, path, f"pub async fn handle() {{ tokio::task::spawn_blocking(|| {call} ); }}\n")
    write(root, "tests/restore/clean-room-restore.sh", "#!/bin/sh\n")
    for name in goldens:
        write(root, f"platforms/jvm/common/src/test/resources/menu-goldens/{name}.json", "{}\n")


def asset_contract():
    return '{"commands":[{"name":"asset.plugin.sync","request":{"fields":{"plugin":{"required":true,"type":"string"}}}}]}'


def generic_asset_contract():
    return '{"commands":[{"name":"asset.plugin.sync","request":{"body":"handler-defined"}}]}'


def mutation_tests(root, reopened_ids, check, forensic, goldens, boundaries):
    source_errors = []
    reopened = reopened_ids(root, source_errors) or {"mutation-fixture"}
    with tempfile.TemporaryDirectory() as temporary:
        fixture_root = Path(temporary)
        fixture(fixture_root, reopened, forensic, goldens, boundaries)
        if check(fixture_root):
            return ["conforming fixture was rejected"]
        cases = [
            ("prior-items-have-probes", "contracts/truth-probe-mapping.json", '{"items":[]}'),
            ("old-runtime-shape-rejected", "crates/lkjmc-daemon/src/app.rs", "Arc<Mutex<Box<dyn RuntimeAdapter>>>\n"),
            ("generic-schema-rejected", "contracts/commands/asset-01.json", None),
            ("generic-schema-rejected", "contracts/commands/asset-01.json", generic_asset_contract()),
            ("payload-consumers-required", "crates/lkjmc-cli/src/client.rs", "pub fn call() {}\n"),
            ("payload-consumers-required", "crates/lkjmc-cli/src/client.rs", 'command_registry::validate_body(command, &body)\ndaemon_command("asset.plugin.sync", json!({}))\n'),
            ("menu-goldens-required", "platforms/jvm/common/src/test/resources/menu-goldens/root.json", None),
            ("doc-source-paths-required", "docs/owner.md", "`scripts/missing.py`\n"),
            ("contracts-size-detected", "scripts/check-docs.py", "def check_state_sources():\n    return ROOT / 'state'\n"),
            ("reactor-blocking-detected", boundaries[0][0], f"pub async fn handle() {{ {boundaries[0][1]} }}\n"),
            ("reactor-blocking-detected", boundaries[1][0], f"pub async fn handle() {{ {boundaries[1][1]} }}\n"),
            ("restore-drill-required", "tests/restore/clean-room-restore.sh", None),
        ]
        failures = []
        for probe, path, value in cases:
            fixture(fixture_root, reopened, forensic, goldens, boundaries)
            target = fixture_root / path
            target.unlink() if value is None else write(fixture_root, path, value)
            if probe not in {name for name, _ in check(fixture_root)}:
                failures.append(f"mutation escaped {probe}: {path}")
        return failures
