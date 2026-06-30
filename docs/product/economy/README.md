# Economy

## Purpose

This area owns point balances, item exchange, shop catalog behavior, and
player-facing economy UX.

## Table of contents

- [Exchange](exchange.md)
- [Shop catalog](shop-catalog.md)

## Current status

Points, balances, shop listing, `minecraft-item` delivery, daemon exchange
rates, default catalog seeding, the Paper `/exchange` command, shop menu UX, and
playable smoke proof for seeded shop purchase plus cobblestone exchange are
implemented.

## Contract

PostgreSQL stores balances, ledgers, rates, exchange events, purchases, and shop
catalog truth. Java adapters own safe inventory mutation and item delivery on the
player scheduler. Daemon handlers own balances, idempotency, rates, and audit.
