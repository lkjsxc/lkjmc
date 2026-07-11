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

## Recovery sequence

1. Obtain independent review of this proposal.
2. Add the narrow checker task and dependency edge in the controller package.
3. Reseal and rebuild controller state from preserved evidence.
4. Complete, review, and integrate the checker task.
5. Rerun `D-VERIFY`, then `DOC-GATE`.

## Non-goals

This amendment does not weaken documentation proof, reclassify a failed check
as passed, or permit product implementation before `DOC-GATE`.
