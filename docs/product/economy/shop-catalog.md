# Shop catalog

## Purpose

This document defines the default point shop catalog and refund-safe purchase UX.

## Status

implemented

Implemented: playable bootstrap seeding, admin seed/status menu action, strict Java
material validation before enablement, balance-rich lore tests, and durable
refund calls for failed Paper delivery.

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

- `minecraft-item`: material and amount delivered by the Paper adapter.
- `adventure`: purchase and start a catalog adventure by `adventureId`.

Unsupported executors, invalid materials, invalid amounts, disabled catalog
items, unaffordable rows, and missing daemon dependencies stay disabled or fail
before point deduction.

## Purchase flow

Deterministic item purchase may execute without confirmation when lore shows the
price, balance, and post-purchase balance and delivery is refund-safe. Paper
sends a correlation id, performs scheduler-safe inventory delivery after daemon
success, and calls the daemon refund command with the same id if delivery fails.
Adventure purchases keep confirmation because they start temporary runtime
state.

## Error mapping

Purchase copy maps exact classes: insufficient points, item not found,
unsupported delivery, invalid material, delivery failed and refunded, database
unavailable, daemon auth failed, duplicate adventure, adventure start failed,
catalog not seeded, and schema mismatch. Generic denial text is not used when a
typed code exists.

## Verification

Core tests compare default buy prices to exchange sell values. Store tests cover
idempotent seeding and purchase/refund ledgers. Java menu tests cover balance
lore, unaffordable shortfall, invalid material, and exact disabled reasons.
