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
- `reviewState`, `reviewedAtCommit`, and `action`;
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

Every nonempty source or check path must exist. Owner lanes replace pending entries
with `changed`, `moved`, `split`, or `confirmed` actions and add follow-up task
IDs for contradictions. A later checker validates the inventory; it is not added
by this inventory task.
