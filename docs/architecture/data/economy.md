# Economy data

## Purpose

This document owns target PostgreSQL data contracts for exchange and catalog
validation.

## Current status

Points and shop tables exist. Exchange rates, exchange events, default catalog
seeding, and anti-arbitrage validation are target behavior until implemented.

## Exchange rates

`economy_exchange_rates` stores material, title key, category, points per item,
minimum amount, enabled state, metadata, and timestamps. `COBBLESTONE` must be
seeded or configured at exactly one point per item.

## Exchange events

`economy_exchange_events` stores player UUID, rate id, material, amount,
points delta, unique correlation id, metadata, and timestamp. Duplicate
correlation ids return the prior result without granting again.

## Catalog validation

Default shop rows use the existing shop item model with `minecraft-item`
delivery metadata. Validation must reject default buy prices that are less than
or equal to any configured sell value for the same material and amount.

## Store boundary

Rust store helpers own SQL and transactions. Daemon handlers use typed helpers;
JVM adapters never inspect raw SQL rows.
