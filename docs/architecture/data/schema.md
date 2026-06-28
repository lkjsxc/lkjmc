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
- `shop_items`
- `shop_purchases`
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

## Migration rule

Migrations are append-only files under `migrations/` and are listed in
`crates/lkjmc-store/src/migrate.rs`. A feature is not durable until both the SQL
migration and typed store helpers exist.

## Presence

Instance presence is implemented by `instance_presence`. It stores latest
heartbeat time, player count when known, readiness, empty timing, suspend and
wake timestamps, and metadata for autosuspend planning.

## Claims

Chunk claims are implemented by `player_claims`, `claim_chunks`, and
`claim_trusts` as described in [claims.md](claims.md).

## Temporary adventures

Temporary adventure data is implemented by `temporary_instances`,
`adventure_sessions`, `adventure_participants`, `adventure_cleanup_events`, and
`temporary_transfer_intents`. Runtime lifecycle and transfer intent commands use
these tables; live purchase orchestration is still separate.
