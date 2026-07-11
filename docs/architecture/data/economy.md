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
points delta, unique correlation id, metadata, and timestamp. Balance, ledger,
and event commit together. Reconciliation reads the settled event by player and
correlation without mutation; an absent result is not proof after an ambiguous
transport failure.

## Catalog validation

Default shop rows use the existing shop item model with `minecraft-item` or
`adventure` delivery metadata. Validation must reject default buy prices that are
less than or equal to any configured sell value for the same material and amount.
`shop_purchases` records a unique correlation and immutable catalog settlement:
item id, title, price, and delivery metadata. A replay returns settlement facts
but no deliverable payload and no refund eligibility. Adventure catalog requests
pass that correlation into the adventure session and its ledger spend, so replay
cannot open or charge a fresh session. Achievement rewards use their own reward
correlation and cannot collide with the source ledger entry.

## Store boundary

Rust store helpers own SQL and transactions. Daemon handlers use typed helpers;
JVM adapters never inspect raw SQL rows.
