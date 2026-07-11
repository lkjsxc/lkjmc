# Shop catalog

## Purpose

This document defines the default point shop catalog and refund-safe purchase UX.

## Status

implemented

Implemented: playable bootstrap seeding, admin seed/status menu action, daemon
material and positive-amount validation before settlement, balance-rich lore
tests, and truthful Paper delivery outcomes.

## Visibility

The default catalog is seeded through real paths: bootstrap apply, playable
Compose setup, admin economy maintenance, and CLI `lkjmc shop seed-defaults`.
A valid empty catalog is distinct from daemon failure. Normal players see a true
empty-catalog state. Admins with economy permission see a real seed action or a
precise disabled reason.

## Item lore

Every purchasable row shows item name, category, material, amount, price,
current balance, balance after purchase, shortfall when unaffordable, delivery
executor, and exact disabled reason. The economy menu must not keep a row whose
only purpose is clicking to display point balance. Balance appears passively in
shop lore, profile summaries, the action bar, and `/points` if command docs keep
that command.

## Delivery executors

- `minecraft-item`: daemon validates a supported Minecraft material and positive
  amount before settlement; Paper delivers the settled item on its player scheduler.
- `adventure`: the correlated adventure session snapshots the catalog delivery.

Unsupported executors, invalid materials, invalid amounts, disabled catalog
items, unaffordable rows, and missing daemon dependencies stay disabled or fail
before point deduction. Invalid metadata never depends on a later refund.

## Purchase flow

Item settlement records immutable item, price, and delivery facts. A replay
returns that snapshot without delivery or refund eligibility before any mutable
catalog lookup. Paper reports item completion only after inventory delivery; a
confirmed safe refund is reported only after its daemon result, while partial or
unknown delivery stays contained.

Adventure settlement records the adventure, price, and target instance under the
outer correlation. A replay resolves that session before catalog lookup and
never starts or charges another session, even if the current catalog price or
delivery changed. Only the localized Adventure EULA confirmation may supply a
true `acceptMinecraftEula`; absent or false shop consent returns bodyless,
non-retryable `adventure.confirmation_required` before session or point work.
After a successful transfer-intent, Paper can only report `transfer-pending`:
plugin-message delivery is not a confirmed transfer and never reports purchase
completion. Intent failure is contained.

## Error mapping

Purchase copy maps exact classes: insufficient points, item not found,
unsupported delivery, invalid material, confirmed refund, contained delivery,
settled replay, transfer pending, database unavailable, daemon auth failed,
duplicate adventure, adventure start failed, catalog not seeded, and schema
mismatch. Generic denial text is not used when a typed code exists.

## Verification

Core tests compare default buy prices to exchange sell values. PostgreSQL-gated
store tests cover immutable replay after catalog mutation and no debit for
invalid item metadata. Daemon and Paper tests cover replay plus pending and
contained transfer wording; Java menu tests cover balance lore and disabled
reasons.
