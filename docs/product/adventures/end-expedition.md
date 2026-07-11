# End expedition

## Purpose

This product contract describes the `end-expedition` catalog entry: a
points-purchased pristine End challenge implemented through temporary Folia
instances.


## Status

implemented

## User flow

A player opens Temporary Adventures, selects End Expedition, reviews cost, party
size, time limit, loot rules, risk rules, and explicit localized Minecraft EULA
acceptance, then selects the action that says it accepts that EULA and starts the
adventure. The daemon validates points, deducts points exactly once, creates an adventure
session, creates a temporary Folia instance, and starts it. Velocity registration
and participant transfer are withdrawn pending trusted identity/session
attestation.

## Data contract

The daemon records adventure id, session id, buyer, participants, point ledger
entry, temporary instance id, state, start deadline, hard stop deadline, refund
state, and audit ids. The generic daemon purchase command spends points, creates
the adventure session, creates the temporary instance, and records participants
in one PostgreSQL transaction.

## Runtime rules

The backend is hidden from normal server lists, uses a unique port, generates a
fresh world directory, installs the `lkjmc` Paper plugin, configures Velocity
forwarding, and has aggressive autosuspend and cleanup policy.

## Failure behavior

If purchase validation, point spend, session creation, temporary instance
creation, or generated world creation fails, the command returns an error and no
points are deducted. If process start or readiness fails after points are deducted, the daemon refunds
through the ledger,
marks the session failed, and audits the transition. Players see a localized
failure, not a live purchase success.

## Consent boundary

Only the localized `adventures-end-confirm` GUI action may originate
`acceptMinecraftEula: true`. Every EULA-gated entry returns the same bodyless,
non-retryable `adventure.confirmation_required` response for absent or false
consent before a connection, plan, purchase, or effect.

| Public entry | Absent or false result |
| --- | --- |
| `adventure.purchase`, `adventure.end.purchase` | shared response before `with_connection` |
| `temporary.instance.create` | shared response before `with_connection` |
| `instance.create.plan`, `instance.create` for an EULA kind | shared response before planning |
| `bootstrap.plan`, `bootstrap.status`, `bootstrap.doctor`, `bootstrap.apply` | shared response before planning or effects |
| `player.shop.purchase` adventure and alternate adventure executors | shared response before nested purchase |
| CLI and daemon direct paths | forwarded unconfirmed request; no local substitute |
| Temporary Adventures Java confirmation | withdrawn with Java daemon menus |

CLI, direct, and admin bodies omit consent. A shop delegate may copy a true
caller value but never creates one. The response has no body and is never
retryable.

## Minecraft surfaces

Java `/endexpedition`, Temporary Adventures menus, transfer intents consumed by
Velocity, and return-to-hub behavior are withdrawn pending trusted
identity/session attestation. Daemon records do not make a player transfer or
Java menu action occur.

## Current status

Adventure session and temporary instance tables, typed store helpers, explicit
daemon temporary instance runtime commands, cleanup worker, catalog purchase,
startup, and refund on startup/readiness failure exist. Java commands, menus,
Velocity registration, transfer, and automatic player return are withdrawn.

## Shop delivery contract

Shop catalog item `adventure-end-expedition` uses the generic `adventure`
delivery executor with `adventureId=end-expedition`. The shop path must not run a
generic item purchase and an adventure purchase for one click. It copies a true
`acceptMinecraftEula` only from an explicit EULA-confirmation action; it never
asserts consent for `/buy` or another direct request. Missing or false consent
returns `adventure.confirmation_required` before any session or point purchase.
Unsupported delivery metadata is rejected before any point deduction.

## Shop status

Daemon purchase and refund paths are live. Java return commands, confirmation
menus, and shop delivery executors are withdrawn; unconfirmed daemon requests
return the shared confirmation response.
