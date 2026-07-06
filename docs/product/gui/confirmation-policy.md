# Confirmation policy

## Purpose

This contract defines when inventory routes, commands, and docs-adjacent
surfaces may ask a player or operator to confirm an action.

## Status

planned

## Required confirmations

A confirmation is required only when an action deletes durable state, overwrites
a named durable object, changes security or moderation state, affects other
players, starts temporary infrastructure, spends points on a non-refund-safe
side effect, stops or force-mutates a server, or bypasses a safety precondition.

Examples include home or warp delete, home or warp location overwrite, claim or
report delete, party leave, server stop, restart, delete, force delete,
create-and-start, paid Nether or End random teleport, adventure purchase, role
changes, token rotation, bans, mutes, warnings, and audited force actions.

## Prohibited confirmations

A confirmation must not appear for Back, Parent Directory, Main Menu, Close,
Refresh, route navigation, documentation browsing, status display, current
balance display, reversible preferences, zero-cost overworld random teleport,
ordinary home teleport, idempotent daily or kit rewards, achievement claims, or
deterministic refund-safe item purchases.

## Document reason vocabulary

Menu documents use these `confirmation` reason tokens:

- `deletes-durable-state`
- `overwrites-named-durable-state`
- `creates-durable-world-state`
- `writes-named-durable-state`
- `stops-server`
- `forceful-server-mutation`
- `starts-durable-resources`
- `starts-temporary-infrastructure`
- `affects-other-players`
- `changes-moderation-state`
- `paid-dimension-change`

## Screen content

Confirmation metadata carries the action id, selected object id, display name,
cost, refund rule, target server or world, force flag, active player count, and
preconditions. Display text never determines the effect. Cancel is true Back and
preserves route history unless stack repair is required.

## Surface size

Simple destructive confirmations use 27 slots: info `4`, confirm `11`, cancel
`15`, and close `26`. Rich confirmations that need context may use 54 slots with
the shared chrome and the same confirmation reason vocabulary.

## Verification

Pure menu tests enumerate confirmation routes and map each to a reason token.
Negative tests prove safe routes do not open confirmation screens.
