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
