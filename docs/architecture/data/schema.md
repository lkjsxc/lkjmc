# Schema

## Purpose

This document names current durable tables and their ownership.

## Current tables

- `schema_migrations`
- `nodes`
- `instances`
- `instance_observations`
- `instance_events`
- `instance_ports`
- `templates`
- `jar_assets`
- `jar_downloads`
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
- `commands`
- `audit_events`
- `outbox_events`

## Migration rule

Migrations are append-only files under `migrations/` and are listed in
`crates/lkjmc-store/src/migrate.rs`. A feature is not durable until both the SQL
migration and typed store helpers exist.

## Next target

Chunk claims will add the tables described in [claims.md](claims.md). They are
not present in the current migration set.
