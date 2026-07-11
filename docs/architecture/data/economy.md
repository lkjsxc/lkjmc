# Economy data

## Purpose

This document owns target PostgreSQL data contracts for exchange and catalog
validation.


## Status

implemented

## Current status

Points, shop, exchange-rate, and exchange-event tables exist. Default catalog
seeding and anti-arbitrage validation are implemented in Rust core/store code.

## Exchange rates

`economy_exchange_rates` stores material, title key, category, points per item,
minimum amount, enabled state, metadata, and timestamps. `COBBLESTONE` must be
seeded or configured at exactly one point per item.

## Exchange events

`economy_exchange_events` stores player UUID, rate id, material, amount,
points delta, unique correlation id, metadata, and timestamp. Duplicate correlation ids return the prior result without granting again. The
balance mutation, ledger row, and exchange event commit atomically, and durable
reconciliation reads those facts by correlation without mutating a settled row.

## Catalog validation

Default shop rows use the existing shop item model with `minecraft-item` or
`adventure` delivery metadata. Validation must reject default buy prices that are
less than or equal to any configured sell value for the same material and amount.
Adventure delivery records the purchase idempotently and must not create a second
point spend outside the adventure transaction. Achievement rewards derived from an exchange or purchase event use their own
reward correlation so they cannot collide with the source ledger entry. Purchase
requests may name only a catalog item: price, reward, and delivery facts are
loaded from PostgreSQL and never accepted from a client payload.

## Store boundary

Rust store helpers own SQL and transactions. Daemon handlers use typed helpers;
JVM adapters never inspect raw SQL rows.
