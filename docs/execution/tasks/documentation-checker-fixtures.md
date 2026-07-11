# Documentation checker fixtures

## Purpose

This is the committed executable mirror of the proposed `D-DOC-CHECK` packet.
It proves each documentation rule fails through the intended checker only.

## Literal task write manifest

`D-DOC-CHECK` may write only:

- `scripts/check-docs.py`;
- `scripts/check-doc-coverage.py`;
- `docs/repository/contract-checks.md`;
- `docs/execution/documentation-coverage.md`;
- `docs/execution/documentation-coverage/execution.json`;
- `docs/execution/documentation-coverage/repository.json`.

## Packet

Run this shell block from the repository root after the checker task exists.

```sh
set -eu
TMP=$(mktemp -d)
trap 'git worktree remove --force "$TMP" 2>/dev/null || true; rm -rf "$TMP"' EXIT
git worktree add --detach "$TMP" HEAD
reset() { git -C "$TMP" restore --source=HEAD --staged --worktree -- .; git -C "$TMP" clean -fd; }
expect_docs() { (cd "$TMP" && python3 scripts/check-docs.py) >"$TMP/out" 2>&1 && exit 1; grep -F "$1" "$TMP/out"; }
expect_coverage() { (cd "$TMP" && python3 scripts/check-doc-coverage.py) >"$TMP/out" 2>&1 && exit 1; grep -F "$1" "$TMP/out"; }
expect_lines() { (cd "$TMP" && python3 scripts/check-lines.py) >"$TMP/out" 2>&1 && exit 1; grep -F "$1" "$TMP/out"; }

mv "$TMP/docs/architecture/assets/README.md" "$TMP/docs/architecture/assets/README.off"
expect_docs 'exactly one README.md'; reset

cat >"$TMP/docs/architecture/assets/unindexed.md" <<'EOF'
Fixture

## Purpose

fixture

## Status

implemented
EOF
expect_docs 'missing link to unindexed.md'; reset

printf '\n[fixture]''(missing.md)\n' >>"$TMP/docs/README.md"
expect_docs 'broken link missing.md'; reset

python3 - "$TMP/docs/state/control-plane.md" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read(); open(p,'w').write(s.replace('`crates/lkjmc-core/src/command_registry.rs`','`crates/absent.rs`',1))
PY
expect_docs 'missing source path crates/absent.rs'; reset

python3 - "$TMP/docs/architecture/runtime/adapters.md" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read(); open(p,'w').write(s.replace('## Status\n\nimplemented','## Status\n\ninvalid-fixture',1))
PY
expect_docs 'invalid status invalid-fixture'; reset

yes fixture | head -n 201 >"$TMP/docs/overlong.md"
expect_lines 'docs/overlong.md: 201 lines'; reset

python3 - "$TMP/docs/execution/documentation-coverage/state.json" <<'PY'
import json,sys
p=sys.argv[1]; x=json.load(open(p)); x['entries']=[r for r in x['entries'] if r['path']!='docs/state/control-plane.md']; open(p,'w').write(json.dumps(x))
PY
expect_coverage 'missing coverage docs/state/control-plane.md'; reset

python3 - "$TMP/docs/execution/documentation-coverage/state.json" <<'PY'
import json,sys
p=sys.argv[1]; x=json.load(open(p)); r=next(r for r in x['entries'] if r['path']=='docs/state/control-plane.md'); r['contentHash']='0'*64; open(p,'w').write(json.dumps(x))
PY
expect_coverage 'hash mismatch docs/state/control-plane.md'; reset

python3 - "$TMP/docs/execution/documentation-coverage/state.json" <<'PY'
import json,sys
p=sys.argv[1]; x=json.load(open(p)); r=next(r for r in x['entries'] if r['path']=='docs/state/control-plane.md'); r['sourceEvidence']=['crates/absent.rs']; open(p,'w').write(json.dumps(x))
PY
expect_coverage 'missing evidence path crates/absent.rs'; reset

python3 - "$TMP/docs/execution/documentation-coverage/state.json" <<'PY'
import json,sys
p=sys.argv[1]; x=json.load(open(p)); r=next(r for r in x['entries'] if r['path']=='docs/state/control-plane.md'); r['action']='invalid-fixture'; open(p,'w').write(json.dumps(x))
PY
expect_coverage 'invalid action invalid-fixture'; reset

python3 - "$TMP/docs/execution/documentation-coverage/state.json" <<'PY'
import json,sys
p=sys.argv[1]; x=json.load(open(p)); r=next(r for r in x['entries'] if r['path']=='docs/state/control-plane.md'); r['reviewedAtCommit']='0000000'; open(p,'w').write(json.dumps(x))
PY
expect_coverage 'invalid review commit 0000000'; reset

python3 - "$TMP/docs/state/control-plane.md" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read(); open(p,'w').write(s.replace('`crates/lkjmc-core/src/command_registry.rs`','`none`',1))
PY
expect_coverage 'implemented capability lacks source evidence'; reset

python3 - "$TMP/docs/state/control-plane.md" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read(); open(p,'w').write(s.replace('`crates/lkjmc-daemon/src/tests/command_registry_tests.rs`','`none`',1))
PY
expect_coverage 'implemented capability lacks deterministic proof'; reset

(cd "$TMP" && python3 scripts/check-docs.py && python3 scripts/check-doc-coverage.py && python3 scripts/check-lines.py)
test -z "$(git -C "$TMP" status --porcelain=v""1)"
```

## Hardening packet

`D-DOC-CHECK-HARDEN` uses the same write manifest and this isolated fixture:

```sh
set -eu
TMP=$(mktemp -d)
trap 'git worktree remove --force "$TMP" 2>/dev/null || true; rm -rf "$TMP"' EXIT
git worktree add --detach "$TMP" HEAD
reset() { git -C "$TMP" restore --source=HEAD --staged --worktree -- .; git -C "$TMP" clean -fd; }
refresh() { python3 - "$TMP/docs/execution/documentation-coverage/state.json" "$TMP/docs/state/surfaces.md" <<'PY'
import hashlib,json,sys
p,path=sys.argv[1:]; x=json.load(open(p)); r=next(r for r in x['entries'] if r['path']=='docs/state/surfaces.md'); r['contentHash']=hashlib.sha256(open(path,'rb').read()).hexdigest(); open(p,'w').write(json.dumps(x))
PY
}
expect() { (cd "$TMP" && python3 scripts/check-doc-coverage.py) >"$TMP/out" 2>&1 && exit 1; grep -F "$1" "$TMP/out"; }

python3 - "$TMP/docs/state/surfaces.md" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read(); open(p,'w').write(s.replace('`cargo test -p lkjmc-discord`','`https://evidence.invalid/discord-proof`',1))
PY
refresh; expect 'invalid deterministic proof https://evidence.invalid/discord-proof'; reset

python3 - "$TMP/docs/state/surfaces.md" <<'PY'
import sys
p=sys.argv[1]; s=open(p).read(); open(p,'w').write(s.replace('`cargo test -p lkjmc-discord`','`not-a-real-deterministic-proof`',1))
PY
refresh; expect 'invalid deterministic proof not-a-real-deterministic-proof'; reset

(cd "$TMP" && python3 scripts/check-doc-coverage.py)
test -z "$(git -C "$TMP" status --porcelain=v""1)"
```

## Boundary

The proposed tasks may only make the fixture expectations true. They may not
loosen a fixture, suppress an error, or change product behavior.
