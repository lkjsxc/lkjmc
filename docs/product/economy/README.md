# Economy

## Purpose

This area owns point balances, item exchange, shop catalog behavior, and
player-facing economy UX.

## Table of contents

- [Exchange](exchange.md)
- [Shop catalog](shop-catalog.md)

## Current status

Points, balances, shop listing, and `minecraft-item` delivery foundations exist.
Cobblestone exchange, default catalog seeding, anti-arbitrage validation, and
refund-on-failure exchange UX are target behavior until implemented and verified.

## Contract

PostgreSQL stores balances, ledgers, rates, exchange events, purchases, and shop
catalog truth. Java adapters own safe inventory mutation and item delivery on the
player scheduler. Daemon handlers own balances, idempotency, rates, and audit.
