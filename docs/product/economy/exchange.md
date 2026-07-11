# Item exchange

## Purpose

This document defines item-to-points exchange behavior.


## Status

implemented

## Current behavior

The source implementation adds daemon exchange rates, idempotent exchange
commits, and a Paper `/exchange <material> <amount|all>` command that removes
items before commit and refunds them on daemon failure. The opt-in playable
smoke covers the cobblestone exchange path when its prerequisites are enabled.

## Required behavior

`COBBLESTONE` exchanges at exactly one point per block. Additional sell rates
are configurable, disabled by default unless seeded intentionally, and must not
create buy/sell profit loops with shop prices.

## Flow

1. The Paper adapter counts eligible items in the player's inventory.
2. The adapter requests a daemon quote when rate data is needed.
3. The adapter removes the exact items on the player scheduler.
4. The adapter commits material, amount, player UUID, and correlation id.
5. The daemon validates the rate, ledger grant, and exchange event in one
   transaction.
6. A replay returns the settled event without a second grant or reward.
7. On every failed commit response, Paper immediately reconciles the same
   correlation. A committed result reports success; a confirmed absent result
   may restore items only when the commit response was definitive.
8. A timeout, failed reconciliation, or failed inventory restoration is
   contained: Paper neither restores or drops items nor claims success. This
   prevents simultaneous points and items.

## Diagnostics

Insufficient items, disabled material, daemon missing, auth failure, database
failure, duplicate correlation, confirmed restoration, and contained ambiguity
are typed product states. They must not expose tokens, database URLs, raw JSON,
or stack traces.
