# Schema

## Purpose

This document names the first durable tables and their ownership.

## Core tables

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
- `player_pending_teleports`
- `player_mail_messages`
- `commands`
- `audit_events`
- `outbox_events`

## Current status

Initial SQL migrations implement the core, instance, jar asset, player profile,
audit, command, outbox, UI settings, party invite, shop, pending teleport, and
player mail schema foundation. Later feature slices may add columns and tables
as their owner docs require.
