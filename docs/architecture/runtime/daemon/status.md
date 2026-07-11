# Status and doctor

## Purpose

This document defines the daemon health contract for operators and automation.


## Status

implemented

## Implemented status body

`status` returns compact JSON with:

- `daemon`, `startedAtUnixSeconds`, and `uptimeSeconds`;
- `database.configured`, `database.connected`, and a sanitized error when a
  configured database cannot be reached or counted;
- `counts.instances`, `counts.activeSessions`, and `counts.jarAssets` when
  PostgreSQL tables are available;
- `roots.config`, `roots.data`, `roots.log`, and `roots.jar`;
- `socket.path`;
- `http.enabled` plus `http.address` when enabled;
- `runtime.adapter` and `runtime.capabilities` for the selected adapter;
- `reconciler.enabled`.

`lkjmc status` prints a human summary by default and preserves the same compact
body with `--json`.

## Implemented doctor checks

`doctor` succeeds only when safe dependency checks pass. It checks that config
loading was intentional, roots are absolute paths with usable ancestors, the
socket parent is a directory, HTTP configuration is enabled or intentionally
disabled, the runtime adapter is selectable, and the configured database can be
reached. Database URLs and secrets are sanitized from errors.

## Bootstrap status

`bootstrap.status --json` includes instance state, installed plugin state,
current bootstrap plan outcome, diagnostics, planned effects, and a `connection`
object with Java bind host, port, public hosts, preferred public host, display
socket, and next connection text. Optional feature withdrawals must be visible
as diagnostics; enabled feature failures must be blocking diagnostics.

## Current boundary

`status` reports database counts only when migrations have made the tables
available. It does not perform write probes against root directories.

## Menu data shape assertions

The daemon Rust shape test asserts these menu response fields with real seeded
PostgreSQL rows where list assertions would otherwise be empty:

| Command | Asserted fields |
| --- | --- |
| `instance.list` | `instances[]`: `id`, `kind`, `desiredState`, `observedState`, `healthy`, `connectHost`, `connectPort`, `proxyRegistrationDesired`, `proxyRegistered`, `joinable`, `joinDisabledReason`, `presence.playerCount` |
| `player.home.list` | `homes[]`: home name in `home`, `serverId`, `location.world`, `location.x`, `location.y`, `location.z` |
| `player.home.get` | `found`, home name in `home`, `serverId`, `location.world`, `location.x`, `location.y`, `location.z` |
| `player.warp.list` | `warps[]`: warp name in `warp`, `serverId`, `location.world`, `location.x`, `location.y`, `location.z` |
| `player.shop.list` | `items[]`: `id`, `titleKey`, `category`, `pricePoints`, `deliveryKind`, `deliveryAvailable`, `disabledReason`, `delivery.executor`, `delivery.material`, `delivery.amount` |
| `player.achievements.list` | `achievements[]`: `id`, `titleKey`, `descriptionKey`, `categoryPath`, `iconMaterial`, `current`, `required`, `state`, `hidden`, `claimable`, `rewardClaimed`, `rewards` |
| `player.random-teleport.quote` | `profileId`, `targetEnvironment`, `costPoints`, `balance`, `cooldownSeconds`, `cooldownRemainingSeconds`, `minRadius`, `maxRadius`, `maxAttempts`, `confirmationRequired`, `enabled`, `canAfford`, `allowedWorlds`, `worldCandidates` |
| `player.settings.get` | `playerUuid`, `language`, `hudEnabled`, `menuEnabled` |
| `player.points.balance` | `playerUuid`, `balance` |
| `player.kit.list` | `kits[]`: `id`, `titleKey`, `rewardPoints`, `cooldownHours` |
| `player.vote.list` | `links[]`: `id`, `titleKey`, `url`, `sortOrder` |
| `player.mail.inbox` | `messages[]`: `id`, `senderName`, `body`, `read` |
| `player.report.list` | `reports[]`: `id`, `reporterUuid`, `targetUuid`, `serverId`, `reason`, `status` |
| `player.daily.status` | `claimedToday`, `points` |
| `player.party.info` | `found`, `name`, `role` |
| `adventure.catalog.list` | `adventures[]`: `id`, `titleKey`, `iconMaterial`, `pricePoints`, `maxPartySize`, `enabled` |
| `claim.list` | `claims[]`: `id`, `ownerUuid`, `ownerName`, `name`, `chunkCount` |

## Source owners

- Dispatch: `crates/lkjmc-daemon/src/dispatch.rs`.
- Status implementation: `crates/lkjmc-daemon/src/commands/status_api.rs`.
- Doctor implementation: `crates/lkjmc-daemon/src/commands/doctor_api.rs`.
- Runtime state: `crates/lkjmc-daemon/src/app.rs`.
- CLI rendering: `crates/lkjmc-cli/src/commands_status.rs`.
