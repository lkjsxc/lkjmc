# Command catalog

## Purpose

This generated document lists public daemon command literals from
[contracts/commands.json](../../../../contracts/commands.json).

## Status

implemented

## admin

- `admin.audit.tail` — admin audit rows in admin_api.rs.
- `admin.grant.create` — durable role grants in admin_api.rs.
- `admin.grant.revoke` — grant revocation in admin_api.rs.
- `admin.principal.inspect` — effective admin grants in admin_api.rs.
- `admin.role.list` — role catalog in admin_api.rs.

## adventure

- `adventure.catalog.list` — adventure_api.rs; returns enabled catalog.
- `adventure.end.purchase` — adventure_api.rs; compatibility path delegating.
- `adventure.end.return` — adventure_api.rs; compatibility path delegating to.
- `adventure.purchase` — adventure_api.rs; purchases any enabled catalog.
- `adventure.return` — adventure_api.rs; validates an active adventure.
- `adventure.session.cancel` — adventure_api.rs; admin cancellation for a.
- `adventure.session.get` — adventure_api.rs; returns a player's active.
- `adventure.session.list` — adventure_api.rs; admin status list for recent.

## announcement

- `announcement.create` — announcement_api.rs.
- `announcement.recent` — announcement_api.rs.

## asset

- `asset.plugin.inspect` — asset_api.rs.
- `asset.plugin.list` — asset_api.rs.
- `asset.plugin.sync` — asset_api.rs.
- `asset.server.sync` — asset_api.rs.

## audit

- `audit.tail` — audit read in api.rs.

## bootstrap

- `bootstrap.apply` — bootstrap_api.rs.
- `bootstrap.doctor` — bootstrap_api.rs.
- `bootstrap.plan` — bootstrap_api.rs.
- `bootstrap.status` — bootstrap_api.rs.

## claim

- `claim.create` — claim_create.rs.
- `claim.delete` — claim_create.rs.
- `claim.list` — claim_read.rs.
- `claim.snapshot` — claim_read.rs.
- `claim.trust` — claim_trust.rs.
- `claim.untrust` — claim_trust.rs.

## config

- `config.reload` — config_api.rs.

## core

- `doctor` — health guard in api.rs.
- `status` — daemon status in api.rs.

## discord

- `discord.link.complete` — Complete a Minecraft account link using a one-time code.
- `discord.link.remove` — Remove the Discord caller account link.
- `discord.wake.request` — requests wake-and-join for a linked Discord user

## economy

- `economy.catalog.seed-defaults` — player_exchange_api.rs.

## instance

- `instance.create` — instance_lifecycle.rs; product surfaces must validate a.
- `instance.create.plan` — instance_create.rs; returns startable-create.
- `instance.delete` — instance_lifecycle.rs.
- `instance.heartbeat` — instance_heartbeat.rs.
- `instance.list` — instance_read.rs.
- `instance.logs` — instance_read.rs.
- `instance.restart` — instance_lifecycle.rs.
- `instance.start` — instance_lifecycle.rs.
- `instance.stop` — instance_lifecycle.rs.
- `instance.wake.cancel` — instance_wake_join.rs; cancels the player's live row.
- `instance.wake.cleanup` — instance_wake_join.rs; expires stale live rows.
- `instance.wake.consume` — instance_wake_join.rs; marks a ready row transferred.
- `instance.wake.request` — instance_wake_join.rs; queues a player for a.
- `instance.wake.status` — instance_wake_join.rs; returns durable wake state.

## jar

- `jar.import` — jars.rs.
- `jar.inspect` — jars.rs.
- `jar.list` — jars.rs.
- `jar.prune` — jar_prune.rs.
- `jar.sync` — downloads.rs.

## kit

- `kit.upsert` — player_kit_api.rs.

## player

- `player.achievement.claim` — player_achievements_api.rs.
- `player.achievement.grant` — player_achievements_api.rs.
- `player.achievements.list` — player_achievements_api.rs.
- `player.actionbar.snapshot` — player_actionbar_api.rs.
- `player.daily.claim` — player_daily_api.rs.
- `player.daily.status` — player_daily_api.rs.
- `player.exchange.commit` — player_exchange_api.rs.
- `player.exchange.quote` — player_exchange_api.rs.
- `player.exchange.rates` — player_exchange_api.rs.
- `player.home.delete` — player_homes_api.rs; deletes a named player home.
- `player.home.get` — player_homes_api.rs.
- `player.home.list` — player_homes_api.rs.
- `player.home.set` — player_homes_api.rs.
- `player.inspect` — player_api.rs.
- `player.kit.claim` — player_kit_api.rs.
- `player.kit.list` — player_kit_api.rs.
- `player.link.begin` — Issue a one-time Discord linking code to a Minecraft player.
- `player.link.remove` — Remove the Minecraft player account link.
- `player.load` — player_api.rs.
- `player.mail.inbox` — player_mail_api.rs.
- `player.mail.read` — player_mail_api.rs.
- `player.mail.send` — player_mail_api.rs.
- `player.moderation.ban` — player_moderation_api.rs.
- `player.moderation.mute` — player_moderation_api.rs.
- `player.moderation.status` — player_moderation_api.rs.
- `player.moderation.unban` — player_moderation_api.rs.
- `player.moderation.unmute` — player_moderation_api.rs.
- `player.note.create` — player_note_api.rs.
- `player.note.list` — player_note_api.rs.
- `player.party.accept` — player_party_api.rs.
- `player.party.create` — player_party_api.rs; optional partyName, otherwise.
- `player.party.info` — player_party_api.rs.
- `player.party.invite` — player_party_api.rs.
- `player.party.leave` — player_party_api.rs.
- `player.points.balance` — player_points_api.rs.
- `player.points.top` — player_points_api.rs.
- `player.random-teleport.complete` — player_random_teleport_api.rs.
- `player.random-teleport.history` — player_random_teleport_api.rs.
- `player.random-teleport.quote` — player_random_teleport_api.rs.
- `player.random-teleport.refund` — player_random_teleport_api.rs.
- `player.random-teleport.reserve` — player_random_teleport_api.rs.
- `player.recovery.report` — player_api.rs.
- `player.report.create` — player_report_api.rs.
- `player.report.dismiss` — player_report_api.rs.
- `player.report.list` — player_report_api.rs.
- `player.report.resolve` — player_report_api.rs.
- `player.restore` — player_restore_api.rs.
- `player.session.join` — player_session_api.rs.
- `player.session.leave` — player_session_api.rs.
- `player.settings.get` — player_settings_api.rs.
- `player.settings.hud` — player_settings_api.rs.
- `player.settings.set` — player_settings_api.rs.
- `player.settings.toggle` — player_settings_api.rs.
- `player.shop.list` — player_shop_api.rs.
- `player.shop.purchase` — player_shop_api.rs.
- `player.snapshot` — player_api.rs.
- `player.teleport.request` — player_teleport_api.rs.
- `player.teleport.take` — player_teleport_api.rs.
- `player.transfer.saved` — player_api.rs.
- `player.vote.list` — player_vote_api.rs.
- `player.warning.create` — player_warning_api.rs.
- `player.warning.list` — player_warning_api.rs.
- `player.warp.get` — player_warps_api.rs.
- `player.warp.list` — player_warps_api.rs.
- `player.warp.set` — player_warps_api.rs.

## proxy

- `proxy.registration.report` — Velocity reports actual managed-server registration state.

## security

- `security.daemon-token.plan` — daemon HTTP token rotation in security_api.rs.
- `security.daemon-token.rotate` — daemon HTTP token rotation in security_api.rs.
- `security.daemon-token.status` — daemon HTTP token rotation in security_api.rs.
- `security.daemon-token.verify` — daemon HTTP token rotation in security_api.rs.

## shop

- `shop.item.upsert` — player_shop_api.rs.

## temporary

- `temporary.instance.cleanup` — temporary_api.rs.
- `temporary.instance.create` — temporary_api.rs.
- `temporary.instance.get` — temporary_api.rs.
- `temporary.instance.start` — temporary_api.rs.
- `temporary.instance.stop` — temporary_api.rs.
- `temporary.transfer.intent` — temporary_api.rs.

## vote

- `vote.link.upsert` — player_vote_api.rs.
- `vote.reward` — player_vote_api.rs.

## Verification

`scripts/check-command-docs.py` verifies this catalog, command docs,
and daemon registration tests against `contracts/commands.json`.
