# Schema

## Purpose

This document names current durable tables and their ownership.

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
- `outbox_events`
- `temporary_instances`
- `adventure_sessions`
- `adventure_participants`
- `adventure_cleanup_events`
- `temporary_transfer_intents`
- `wake_join_queue`
- `random_teleports`
- `discord_account_links`
- `admin_roles`
- `admin_grants`
- `admin_audit`

## Migration rule

Migrations are append-only files under `migrations/` and are listed in
`crates/lkjmc-store/src/migrate.rs`. A feature is not durable until both the SQL
migration and typed store helpers exist.

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
`adventure_sessions`, `adventure_participants`, `adventure_cleanup_events`, and
`temporary_transfer_intents`. Runtime lifecycle, transfer intent, and live
purchase commands use these tables.

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
