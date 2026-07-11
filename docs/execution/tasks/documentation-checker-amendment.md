# Documentation checker amendment

## Purpose

This proposal resolves an internal gate contradiction discovered by independent
`D-VERIFY`: the gate requires source, coverage, and capability-evidence
violations to fail, but the current checks do not enforce them.

## Evidence

The verifier proved that `check-docs.py` rejects missing indexes, bad links,
invalid status, and line overflow. It does not reject a stale backticked source
path, an omitted coverage record, or an implemented matrix row without source
and deterministic proof.

## Narrow exception

This verification infrastructure task may write only these exact paths:

- `scripts/check-docs.py` and `scripts/check-doc-coverage.py`;
- `docs/repository/contract-checks.md`;
- `docs/execution/documentation-coverage.md`;
- `docs/execution/documentation-coverage/execution.json`;
- `docs/execution/documentation-coverage/repository.json`;
- `docs/execution/documentation-coverage/architecture.json`.

The architecture shard exception replaces existing directory evidence with
concrete regular-file source traces only. It may not modify Rust, Java, SQL,
contracts, build configuration, runtime configuration, daemon behavior, adapters,
or public product surfaces.

## Exact controller graph patch

Append this task to `manifests/tasks/documentation.json` and change `D-VERIFY`
to depend on `D-DOC-CHECK` instead of `D-STATE`:

```json
{"id":"D-DOC-CHECK","phase":"documentation","owner":"docs-checker","mode":"shared","dependsOn":["D-STATE"],"packet":"tasks/documentation/documentation-checker.md#d-doc-check","writes":["scripts/check-docs.py","scripts/check-doc-coverage.py","docs/repository/contract-checks.md","docs/execution/documentation-coverage.md","docs/execution/documentation-coverage/execution.json","docs/execution/documentation-coverage/repository.json","docs/execution/documentation-coverage/architecture.json"],"probes":["stale-source-rejected","coverage-drift-rejected","capability-evidence-rejected","all-proof-violations-rejected","checker-clean-tree"],"externalBlockable":false}
```

## Exact packet contract

Create `tasks/documentation/documentation-checker.md` by copying the committed
[fixture packet](documentation-checker-fixtures.md) verbatim. That packet uses
`mktemp`, detached worktrees, exact fixture targets, intended-checker assertions,
reset, clean-status, and cleanup commands. It requires nonzero exit for:

1. move `docs/architecture/assets/README.md` aside;
2. add a valid but unlinked `docs/architecture/assets/unindexed.md`;
3. add a broken local Markdown link to `docs/README.md`;
4. replace a backticked source path in `docs/state/control-plane.md` with
   `crates/absent.rs`;
5. remove the `docs/state/control-plane.md` coverage record;
6. set a valid owner document status to `invalid-fixture`;
7. replace one implemented matrix source and deterministic-proof cell with
   `none`;
8. add a 201-line Markdown file;
9. replace one coverage hash, evidence path, action, and review commit with
   invalid values, one fixture at a time.

Restore every fixture, require clean status, then run both checkers successfully.
The packet must name the same exact write roots as the graph task.

## Preserved proof

The amendment keeps `D-VERIFY` and all five original probes. `D-DOC-CHECK`
strengthens, never substitutes, its later independent review.

## Original probe mapping

| Original `D-VERIFY` probe | Preserved or strengthened proof |
| --- | --- |
| `docs-mechanical-pass` | Both checkers enforce structural and semantic documentation rules. |
| `violation-tests-pass` | Checker task proves all eight violations reject before verifier reruns them. |
| `semantic-sample-pass` | Remains an independent read-only verifier responsibility. |
| `coverage-equals-tree` | Coverage checker enforces tree, hash, evidence, action, and provenance. |
| `prior-mismatches-open` | Remains an independent read-only verifier responsibility. |

## Controller recovery corrections

The approved rebuild sequence exposed two pre-existing recovery bugs.

- Before `check_review()`, `rebuild()` must set
  `record["evidence"] = str(evidence_path)`.
- Direct integration keeps requiring the current clean `HEAD`. Rebuild instead
  validates historic integration: its recorded head exists and is an ancestor of
  current `HEAD`; each integration commit is an ancestor of that recorded head;
  source commits exist; and the current worktree is clean.

Add regression tests for accepted evidence/review recovery and for rebuilding an
integration after a later commit. These are controller recovery only; they do not
change task semantics or product behavior.

## Exact recovery sequence

1. Obtain independent approval of this exact patch and recovery correction.
2. Edit the controller task manifest, packet, ownership, barrier, and recovery test.
3. Run `python3 control/validate.py` and `python3 control/test_planctl.py`.
4. Run `python3 control/seal.py create`.
5. Run `python3 control/planctl.py rebuild` from preserved evidence.
6. Confirm accepted tasks and recorded external blockers remain preserved.
7. Complete, review, and integrate `D-DOC-CHECK`.
8. Rerun `D-VERIFY`, then `DOC-GATE`.

## Non-goals

This amendment does not weaken documentation proof, reclassify a failed check
as passed, or permit product implementation before `DOC-GATE`.
