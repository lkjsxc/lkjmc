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

## Current status

Initial SQL migrations implement the core, instance, jar asset, player profile,
audit, command, outbox, UI settings, party invite, shop, pending teleport,
player mail, player report, player warning, player note, moderation punishment,
daily reward, announcement, kit, vote link, and vote reward schema foundation. Later feature slices may add columns and tables as their owner docs
require.
