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
5. The daemon validates the rate, records the ledger grant and exchange event,
   and updates the balance in one transaction.
6. A repeated correlation returns the original durable result; it never grants
   points or applies achievement rewards again.
7. The adapter reports success through chat/menu/action bar only after a
   committed result.
8. If removal, refund, or delivery state is ambiguous, the adapter contains the
   operation: it does not drop items or claim success. An operator uses the
   correlation reconciliation command and durable ledger/event facts.

## Diagnostics

Insufficient items, disabled material, daemon missing, auth failure, database
failure, duplicate correlation, and ambiguous inventory state are typed product
states. They must not expose tokens, database URLs, raw JSON, or stack traces.
