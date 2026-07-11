# Documentation coverage

## Purpose

This contract defines the sharded inventory used to prove that every committed
Markdown file is reviewed during the documentation campaign.

## Status

active

## Inventory shape

`documentation-coverage.json` indexes JSON shards in
`documentation-coverage/`. Each entry has:

- `path`, `contentHash`, `role`, `owner`, and current declared `status`;
- `reviewState`, `reviewedAtCommit`, and `action` (`unchanged`, `rewritten`, or
  `added`);
- `sourceEvidence`, `checkEvidence`, `contradictions`, and `followUpTasks`.

The inventory is sharded by top-level documentation area so authored files remain
below the line limit. Entries begin as `pending` until their owner lane audits
them. Empty source evidence means no source claim has been audited yet; it is not
proof of implementation.

## Capability dimensions

Each shipped capability must eventually distinguish:

- contract present;
- implementation present;
- deterministic proof;
- Compose proof;
- live proof;
- degraded behavior;
- external block.

State files may report only dimensions backed by owner documentation and exact
checks. Target and experiment claims remain outside shipped state.

## Evidence rule

Every nonempty source or check path must exist. Each reviewed document records
its own exact source trace. `unchanged` means the reviewed content hash was
retained; `rewritten` means the reviewed document content changed; `added` means
this review introduced the document. Owner lanes add follow-up task IDs for
contradictions. The index stores each known contradiction with affected coverage
paths and task IDs. A later checker validates the inventory; it is not added by
this inventory task.
