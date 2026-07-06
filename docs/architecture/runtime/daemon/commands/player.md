# Player commands

## Purpose

This generated file lists `player` daemon command literals from
[contracts/commands.json](../../../../../contracts/commands.json).

## Status

implemented

## Commands

| Command | Authorization | Surfaces | Summary |
| --- | --- | --- | --- |
| `player.achievement.claim` | open | cli, discord, paper, velocity, web | player_achievements_api.rs. |
| `player.achievement.grant` | open | cli, discord, paper, velocity, web | player_achievements_api.rs. |
| `player.achievements.list` | open | cli, discord, paper, velocity, web | player_achievements_api.rs. |
| `player.actionbar.snapshot` | open | cli, discord, paper, velocity, web | player_actionbar_api.rs. |
| `player.daily.claim` | open | cli, discord, paper, velocity, web | player_daily_api.rs. |
| `player.daily.status` | open | cli, discord, paper, velocity, web | player_daily_api.rs. |
| `player.exchange.commit` | open | cli, discord, paper, velocity, web | player_exchange_api.rs. |
| `player.exchange.quote` | open | cli, discord, paper, velocity, web | player_exchange_api.rs. |
| `player.exchange.rates` | open | cli, discord, paper, velocity, web | player_exchange_api.rs. |
| `player.home.delete` | open | cli, discord, paper, velocity, web | player_homes_api.rs; deletes a named player home. |
| `player.home.get` | open | cli, discord, paper, velocity, web | player_homes_api.rs. |
| `player.home.list` | open | cli, discord, paper, velocity, web | player_homes_api.rs. |
| `player.home.set` | open | cli, discord, paper, velocity, web | player_homes_api.rs. |
| `player.inspect` | open | cli, discord, paper, velocity, web | player_api.rs. |
| `player.kit.claim` | open | cli, discord, paper, velocity, web | player_kit_api.rs. |
| `player.kit.list` | open | cli, discord, paper, velocity, web | player_kit_api.rs. |
| `player.link.begin` | open | paper | Issue a one-time Discord linking code to a Minecraft player. |
| `player.link.remove` | open | paper | Remove the Minecraft player account link. |
| `player.load` | open | cli, discord, paper, velocity, web | player_api.rs. |
| `player.mail.inbox` | open | cli, discord, paper, velocity, web | player_mail_api.rs. |
| `player.mail.read` | open | cli, discord, paper, velocity, web | player_mail_api.rs. |
| `player.mail.send` | open | cli, discord, paper, velocity, web | player_mail_api.rs. |
| `player.moderation.ban` | open | cli, discord, paper, velocity, web | player_moderation_api.rs. |
| `player.moderation.mute` | open | cli, discord, paper, velocity, web | player_moderation_api.rs. |
| `player.moderation.status` | open | cli, discord, paper, velocity, web | player_moderation_api.rs. |
| `player.moderation.unban` | open | cli, discord, paper, velocity, web | player_moderation_api.rs. |
| `player.moderation.unmute` | open | cli, discord, paper, velocity, web | player_moderation_api.rs. |
| `player.note.create` | open | cli, discord, paper, velocity, web | player_note_api.rs. |
| `player.note.list` | open | cli, discord, paper, velocity, web | player_note_api.rs. |
| `player.party.accept` | open | cli, discord, paper, velocity, web | player_party_api.rs. |
| `player.party.create` | open | cli, discord, paper, velocity, web | player_party_api.rs; optional partyName, otherwise. |
| `player.party.info` | open | cli, discord, paper, velocity, web | player_party_api.rs. |
| `player.party.invite` | open | cli, discord, paper, velocity, web | player_party_api.rs. |
| `player.party.leave` | open | cli, discord, paper, velocity, web | player_party_api.rs. |
| `player.points.balance` | open | cli, discord, paper, velocity, web | player_points_api.rs. |
| `player.points.top` | open | cli, discord, paper, velocity, web | player_points_api.rs. |
| `player.random-teleport.complete` | open | cli, discord, paper, velocity, web | player_random_teleport_api.rs. |
| `player.random-teleport.history` | open | cli, discord, paper, velocity, web | player_random_teleport_api.rs. |
| `player.random-teleport.quote` | open | cli, discord, paper, velocity, web | player_random_teleport_api.rs. |
| `player.random-teleport.refund` | open | cli, discord, paper, velocity, web | player_random_teleport_api.rs. |
| `player.random-teleport.reserve` | open | cli, discord, paper, velocity, web | player_random_teleport_api.rs. |
| `player.recovery.report` | open | cli, discord, paper, velocity, web | player_api.rs. |
| `player.report.create` | open | cli, discord, paper, velocity, web | player_report_api.rs. |
| `player.report.dismiss` | open | cli, discord, paper, velocity, web | player_report_api.rs. |
| `player.report.list` | open | cli, discord, paper, velocity, web | player_report_api.rs. |
| `player.report.resolve` | open | cli, discord, paper, velocity, web | player_report_api.rs. |
| `player.restore` | open | cli, discord, paper, velocity, web | player_restore_api.rs. |
| `player.session.join` | open | cli, discord, paper, velocity, web | player_session_api.rs. |
| `player.session.leave` | open | cli, discord, paper, velocity, web | player_session_api.rs. |
| `player.settings.get` | open | cli, discord, paper, velocity, web | player_settings_api.rs. |
| `player.settings.hud` | open | cli, discord, paper, velocity, web | player_settings_api.rs. |
| `player.settings.set` | open | cli, discord, paper, velocity, web | player_settings_api.rs. |
| `player.settings.toggle` | open | cli, discord, paper, velocity, web | player_settings_api.rs. |
| `player.shop.list` | open | cli, discord, paper, velocity, web | player_shop_api.rs. |
| `player.shop.purchase` | open | cli, discord, paper, velocity, web | player_shop_api.rs. |
| `player.shop.refund` | open | paper | player_shop_api.rs; refunds a failed scheduler-side shop delivery by correlation id. |
| `player.snapshot` | open | cli, discord, paper, velocity, web | player_api.rs. |
| `player.teleport.request` | open | cli, discord, paper, velocity, web | player_teleport_api.rs. |
| `player.teleport.take` | open | cli, discord, paper, velocity, web | player_teleport_api.rs. |
| `player.transfer.saved` | open | cli, discord, paper, velocity, web | player_api.rs. |
| `player.vote.list` | open | cli, discord, paper, velocity, web | player_vote_api.rs. |
| `player.warning.create` | open | cli, discord, paper, velocity, web | player_warning_api.rs. |
| `player.warning.list` | open | cli, discord, paper, velocity, web | player_warning_api.rs. |
| `player.warp.get` | open | cli, discord, paper, velocity, web | player_warps_api.rs. |
| `player.warp.list` | open | cli, discord, paper, velocity, web | player_warps_api.rs. |
| `player.warp.set` | open | cli, discord, paper, velocity, web | player_warps_api.rs. |
