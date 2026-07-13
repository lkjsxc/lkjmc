# Schema

## Purpose

This document names current durable tables and their ownership.


## Status

implemented

## Current tables

- `schema_migrations`
- `nodes`
- `instances`
- `instance_observations`
- `instance_presence`
- `instance_events`
- `instance_ports`
- `templates`
- `jar_assets`
- `jar_downloads`
- `assets`
- `asset_downloads`
- `plugin_catalog_entries`
- `instance_plugin_installations`
- `bootstrap_runs`
- `bootstrap_steps`
- `player_identities`
- `player_sessions`
- `player_profile_leases`
- `player_profile_snapshots`
- `player_profile_snapshot_quarantine`
- `player_settings`
- `points_accounts`
- `points_ledger`
- `homes`
- `warps`
- `parties`
- `party_members`
- `party_invites`
- `achievements`
- `player_achievements`
- `achievement_reward_claims`
- `shop_items`
- `shop_purchases`
- `economy_exchange_rates`
- `economy_exchange_events`
- `kit_definitions`
- `player_kit_claims`
- `vote_links`
- `player_vote_rewards`
- `player_pending_teleports`
- `player_mail_messages`
- `player_reports`
- `player_warnings`
- `player_notes`
- `player_punishments`
- `player_daily_claims`
- `announcements`
- `player_claims`
- `claim_chunks`
- `claim_trusts`
- `commands`
- `audit_events`
- `temporary_instances`
- `adventure_sessions`
- `adventure_participants`
- `adventure_cleanup_events`
- `transfer_workflows`
- `item_delivery_workflows`
- `runtime_effect_workflows`
- `workflow_change_feed`
- `workflow_change_archive`
- `workflow_retention_policy`
- `wake_join_queue`
- `random_teleports`
- `discord_account_links`
- `admin_roles`
- `admin_grants`
- `admin_audit`

## Migration rule

Migrations are append-only files under `migrations/` and are listed in
`crates/lkjmc-store/src/migrate.rs`. A feature is not durable until both the SQL
migration and typed store helpers exist. Migration `045` is a destructive
internal cutover: pre-cutover profile rows are quarantined without conversion,
superseded transfer state is dropped, and only typed-envelope snapshots and the
new workflow tables remain writable. Pre-cutover adventure `ready` and `active` rows become pending start intent
because no trusted acknowledgement exists;
the cutover never synthesizes observation or success.

## Revisions and retention

Profile and workflow writes append a globally monotonic change row in the same
transaction. Each aggregate also has a compare-and-swap revision. Active feed
rows are retained for 30 days, archives for 365 days; archive and deletion run
only through the store retention transaction. Consumers resume by durable feed revision. The resume floor is the minimum
active-feed revision because resume returns active rows only. A cursor below
that floor, including one pointing into archive or a deleted range, receives a
typed reload-required result and must perform a bounded full reload.

## Presence

Instance presence is implemented by `instance_presence`. It stores latest
heartbeat time, player count when known, readiness, empty timing, suspend and
wake timestamps, and metadata for autosuspend planning.

## Admin

Admin roles, grants, and audit rows are implemented by `admin_roles`,
`admin_grants`, and `admin_audit` as described in
[../runtime/admin-rbac.md](../runtime/admin-rbac.md).

## Economy

Point exchange data is implemented by `economy_exchange_rates` and
`economy_exchange_events` as described in [economy.md](economy.md). Achievement
reward claims use `achievement_reward_claims`, point-ledger correlation rows, and
mail message rows for the implemented mail reward executor. Other non-point
reward executors remain disabled until their own durable delivery helpers exist.

## Claims

Chunk claims are implemented by `player_claims`, `claim_chunks`, and
`claim_trusts` as described in [claims.md](claims.md).

## Temporary adventures

Temporary adventure data is implemented by `temporary_instances`,
`adventure_sessions`, `adventure_participants`, and `adventure_cleanup_events`.
The superseded temporary transfer table is dropped by migration `045`; transfer
intent exists only in `transfer_workflows`.

## Wake-and-join

Suspended backend wake requests are implemented by `wake_join_queue`. The daemon
records the player and target, wakes the backend, and marks the row ready or
failed before a transfer control may send the player.

## Discord links

Discord account linking uses `discord_account_links` for Discord user id,
Minecraft UUID, verification state, created time, verified time, revoked time,
and metadata. `link_codes` stores one active hashed one-time code per player
with expiry and consumption state. Link-required surfaces must read these tables
instead of faking a Minecraft identity.
