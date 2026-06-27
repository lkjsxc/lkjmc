# Command catalog

## Purpose

This document lists public daemon command literals and their source owners.

## Core and runtime

- `doctor` — health guard in `api.rs`.
- `status` — daemon status in `api.rs`.
- `config.reload` — `config_api.rs`.
- `audit.tail` — audit read in `api.rs`.

## Instances and jars

- `instance.list` — `instance_read.rs`.
- `instance.create` — `instance_lifecycle.rs`.
- `instance.start` — `instance_lifecycle.rs`.
- `instance.stop` — `instance_lifecycle.rs`.
- `instance.restart` — `instance_lifecycle.rs`.
- `instance.delete` — `instance_lifecycle.rs`.
- `instance.logs` — `instance_read.rs`.
- `instance.heartbeat` — `instance_heartbeat.rs`.
- `jar.list` — `jars.rs`.
- `jar.import` — `jars.rs`.
- `jar.inspect` — `jars.rs`.
- `jar.sync` — `downloads.rs`.
- `jar.prune` — `jar_prune.rs`.

## Bootstrap and assets

- `bootstrap.plan` — `bootstrap_api.rs`.
- `bootstrap.apply` — `bootstrap_api.rs`.
- `bootstrap.status` — `bootstrap_api.rs`.
- `bootstrap.doctor` — `bootstrap_api.rs`.
- `asset.server.sync` — `asset_api.rs`.
- `asset.plugin.sync` — `asset_api.rs`.
- `asset.plugin.list` — `asset_api.rs`.
- `asset.plugin.inspect` — `asset_api.rs`.

## Claims

- `claim.create` — `claim_create.rs`.
- `claim.delete` — `claim_create.rs`.
- `claim.list` — `claim_read.rs`.
- `claim.snapshot` — `claim_read.rs`.
- `claim.trust` — `claim_trust.rs`.
- `claim.untrust` — `claim_trust.rs`.

## Player profile, session, and settings

- `player.inspect` — `player_api.rs`.
- `player.load` — `player_api.rs`.
- `player.snapshot` — `player_api.rs`.
- `player.transfer.saved` — `player_api.rs`.
- `player.recovery.report` — `player_api.rs`.
- `player.restore` — `player_restore_api.rs`.
- `player.session.join` — `player_session_api.rs`.
- `player.session.leave` — `player_session_api.rs`.
- `player.settings.get` — `player_settings_api.rs`.
- `player.settings.set` — `player_settings_api.rs`.
- `player.settings.hud` — `player_settings_api.rs`.
- `player.settings.toggle` — `player_settings_api.rs`.

## Homes, warps, teleports, and parties

- `player.home.get` — `player_homes_api.rs`.
- `player.home.list` — `player_homes_api.rs`.
- `player.home.set` — `player_homes_api.rs`.
- `player.warp.get` — `player_warps_api.rs`.
- `player.warp.list` — `player_warps_api.rs`.
- `player.warp.set` — `player_warps_api.rs`.
- `player.teleport.request` — `player_teleport_api.rs`.
- `player.teleport.take` — `player_teleport_api.rs`.
- `player.party.create` — `player_party_api.rs`.
- `player.party.invite` — `player_party_api.rs`.
- `player.party.accept` — `player_party_api.rs`.
- `player.party.info` — `player_party_api.rs`.
- `player.party.leave` — `player_party_api.rs`.

## Economy, rewards, and announcements

- `player.points.balance` — `player_points_api.rs`.
- `player.points.top` — `player_points_api.rs`.
- `player.shop.list` — `player_shop_api.rs`.
- `player.shop.purchase` — `player_shop_api.rs`.
- `shop.item.upsert` — `player_shop_api.rs`.
- `player.kit.list` — `player_kit_api.rs`.
- `player.kit.claim` — `player_kit_api.rs`.
- `kit.upsert` — `player_kit_api.rs`.
- `player.vote.list` — `player_vote_api.rs`.
- `vote.link.upsert` — `player_vote_api.rs`.
- `vote.reward` — `player_vote_api.rs`.
- `player.daily.status` — `player_daily_api.rs`.
- `player.daily.claim` — `player_daily_api.rs`.
- `announcement.create` — `announcement_api.rs`.
- `announcement.recent` — `announcement_api.rs`.
- `player.achievement.grant` — `player_achievements_api.rs`.
- `player.achievements.list` — `player_achievements_api.rs`.

## Mail and moderation

- `player.mail.inbox` — `player_mail_api.rs`.
- `player.mail.read` — `player_mail_api.rs`.
- `player.mail.send` — `player_mail_api.rs`.
- `player.report.create` — `player_report_api.rs`.
- `player.report.list` — `player_report_api.rs`.
- `player.report.resolve` — `player_report_api.rs`.
- `player.report.dismiss` — `player_report_api.rs`.
- `player.warning.create` — `player_warning_api.rs`.
- `player.warning.list` — `player_warning_api.rs`.
- `player.note.create` — `player_note_api.rs`.
- `player.note.list` — `player_note_api.rs`.
- `player.moderation.ban` — `player_moderation_api.rs`.
- `player.moderation.unban` — `player_moderation_api.rs`.
- `player.moderation.mute` — `player_moderation_api.rs`.
- `player.moderation.unmute` — `player_moderation_api.rs`.
- `player.moderation.status` — `player_moderation_api.rs`.

## Verification

`scripts/check-command-docs.py` extracts current command literals from daemon
routers and checks this catalog. Target-only command names are intentionally not
formatted as current literals until routers exist.
