# Documentation coverage

## Purpose

This contract defines the sharded inventory that proves every committed Markdown
file has an owner, current hash, provenance, and review action.

## Status

active

## Inventory shape

`documentation-coverage.json` indexes JSON shards in
`documentation-coverage/`. Every entry records `path`, `contentHash`, `role`,
`owner`, declared `status`, `reviewState`, `reviewedAtCommit`, `action`,
`sourceEvidence`, `checkEvidence`, `contradictions`, and `followUpTasks`.

`contentHash` is SHA-256 of the current UTF-8 file. Review states are `pending`,
`audited`, or `reviewed`; every record names an existing Git review commit.
`reviewState: reviewed` requires a current hash and existing evidence paths. The
complete current action vocabulary is:

- `pending`: inventory record not yet audited; it proves no review result.
- `added`: the recorded review introduced the document.
- `changed`: a historical review changed the document; it is retained without
  retroactively restating its older review convention.
- `confirmed`: a review confirmed the recorded content and evidence.
- `retain-with-boundary`: a review retained the document while recording an
  evidence or behavior boundary.
- `rewritten`: the current campaign's audited content change.
- `unchanged`: the current campaign reviewed and retained the content.

The action describes the review record, not implementation status or live proof.

## Evidence rule

`sourceEvidence` names exact repository owner or source paths. `checkEvidence`
names deterministic check or test paths; a guarded command is recorded only as
live evidence in its owner capability matrix. Every nonempty path must resolve
within the repository and exist. An implemented capability needs both source and
deterministic evidence; an empty source list records process or history, not
implementation proof. Each deterministic proof code span is an existing regular
repository file or a validated local command: `cargo test -p <workspace-package>`
or `docker compose --profile verify run --rm verify` for the configured service.

## Capability dimensions

State matrices distinguish owner contract, source, deterministic proof, guarded
live proof, present limit, and follow-up task. Target and experiment material is
not shipped state. Known contradictions stay in the index with affected paths
and task IDs until their follow-up changes provide stronger evidence.

## Validation

`check-doc-coverage.py` compares the inventory with Git-tracked Markdown, then
validates paths, hashes, evidence, actions, and review commits. It also requires
every implemented state-matrix row to name source and deterministic proof code
spans that satisfy the repository-file or validated-local-command grammar.
Refresh hashes after every documentation edit. Coverage data is inventory
evidence; it does not replace owner documentation or controller state.
