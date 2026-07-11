# Economy

## Purpose

This area owns point balances, item exchange, shop catalog behavior,
achievements, and player-facing progression UX.


## Status

implemented

## Table of contents

- [Exchange](exchange.md)
- [Shop catalog](shop-catalog.md)
- [Achievements](achievements.md)

## Current status

Points, balances, shop listing, `minecraft-item` delivery, adventure delivery,
daemon exchange rates, default catalog seeding, achievement definitions,
progress listing, the Paper `/exchange` command, shop menu UX, and playable
smoke proof for seeded shop purchase plus cobblestone exchange are implemented.

## Contract

PostgreSQL stores balances, ledgers, rates, exchange events, purchases, shop
catalog truth, achievement definitions, and progress. Java adapters own safe
inventory mutation and item delivery on the player scheduler. Daemon handlers own
balances, idempotency, rates, rewards, progress, and audit.

## Outcome, journey, and evidence boundary

A player sees a current balance and an enabled offer, exchanges or purchases
through a correlation-safe path, and receives delivery or an exact failure.
Unavailable catalog data, invalid delivery metadata, insufficient points, and
daemon failure disable or fail before a false success; failed post-charge item
delivery uses the refund path. Ledger, store, and menu tests support these
claims; only the opt-in playable smoke proves an in-game delivery run.
