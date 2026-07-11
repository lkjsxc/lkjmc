# Economy

## Purpose

This area owns daemon/store point balances, exchange rates, shop catalog data,
and achievement definitions.

## Status

implemented

## Table of contents

- [Exchange](exchange.md)
- [Shop catalog](shop-catalog.md)
- [Achievements](achievements.md)

## Current status

Points, balances, daemon exchange rates, default catalog seeding, purchase
ledgers, and achievement definitions are implemented as daemon/store behavior.
Paper `/exchange`, shop menus, inventory delivery, and adventure transfer are
withdrawn pending trusted identity/session attestation.

## Contract

PostgreSQL stores balances, ledgers, rates, exchange events, purchases, catalog
truth, achievement definitions, progress, and audit. Java plugins do not mutate
inventory or call the daemon. A daemon settlement record does not claim a Java
player delivery.

## Evidence boundary

Core and store tests prove ledger and catalog behavior. They do not prove an
in-game exchange, shop view, delivery, or transfer.
