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

Add one documentation-phase task before `D-VERIFY` that may change only:

- `scripts/check-docs.py`;
- a new `scripts/check-doc-coverage.py`;
- documentation checker owner docs and coverage rows.

This is verification infrastructure, not product behavior. It may not modify
Rust, Java, SQL, contracts, build configuration, runtime configuration, daemon
behavior, adapters, or public product surfaces.

## Preserved proof

The amendment keeps `D-VERIFY` and all five original probes. The new task must
make the existing proof stronger by rejecting:

- stale backticked source paths;
- coverage tree, hash, evidence, action, and review-provenance drift;
- an implemented capability row without source and deterministic proof.

`D-VERIFY` must rerun every deliberate violation after the checker task and may
still reject the documentation campaign.

## Controller graph patch

Add `D-DOC-CHECK` in the documentation phase with dependency `D-STATE` and
make `D-VERIFY` depend on `D-DOC-CHECK`. Its only write roots are:

- `scripts/check-docs.py`;
- `scripts/check-doc-coverage.py`;
- `docs/execution/**` and the checker owner documentation it references.

Its probes are `stale-source-rejected`, `coverage-drift-rejected`,
`capability-evidence-rejected`, `all-proof-violations-rejected`, and
`checker-clean-tree`. It must not write product, runtime, configuration,
contract, build, migration, or adapter paths.

## Executable checker contract

The task must run both checkers in a temporary worktree and prove every
violation from `documentation/proof.md` fails for its intended rule: missing
index, unindexed child, broken link, stale source, omitted coverage row, invalid
status, missing implemented capability evidence, and line overflow. It must also
reject coverage hash, evidence, action, and review-provenance drift. The shared
tree must remain clean after each fixture is removed.

## Original probe mapping

| Original `D-VERIFY` probe | Preserved or strengthened proof |
| --- | --- |
| `docs-mechanical-pass` | Both checkers enforce structural and semantic documentation rules. |
| `violation-tests-pass` | Checker task proves all eight violations reject before verifier reruns them. |
| `semantic-sample-pass` | Remains an independent read-only verifier responsibility. |
| `coverage-equals-tree` | Coverage checker enforces tree, hash, evidence, action, and provenance. |
| `prior-mismatches-open` | Remains an independent read-only verifier responsibility. |

## Exact recovery sequence

1. Obtain independent approval of this exact patch.
2. Edit the controller task manifest and add the executable task packet.
3. Run `python3 control/validate.py` and `python3 control/test_planctl.py`.
4. Run `python3 control/seal.py create`.
5. Run `python3 control/planctl.py rebuild` from preserved evidence.
6. Confirm accepted tasks and recorded external blockers remain preserved.
7. Complete, review, and integrate `D-DOC-CHECK`.
8. Rerun `D-VERIFY`, then `DOC-GATE`.

## Non-goals

This amendment does not weaken documentation proof, reclassify a failed check
as passed, or permit product implementation before `DOC-GATE`.
