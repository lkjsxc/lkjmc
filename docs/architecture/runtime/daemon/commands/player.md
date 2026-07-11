# Player commands

## Purpose

This generated file lists `player` daemon command literals from
[contracts/commands.json](../../../../../contracts/commands.json).

## Status

implemented

## Commands

| Command | Authorization | Surfaces | Summary |
| --- | --- | --- | --- |
| `player.achievement.claim` | operator | cli, paper, velocity, web | Execute player achievement claim behavior for the player command family. |
| `player.achievement.grant` | operator | cli, paper, velocity, web | Execute player achievement grant behavior for the player command family. |
| `player.achievements.list` | operator | cli, paper, velocity, web | Execute player achievements list behavior for the player command family. |
| `player.actionbar.snapshot` | operator | cli, paper, velocity, web | Execute player actionbar snapshot behavior for the player command family. |
| `player.daily.claim` | operator | cli, paper, velocity, web | Execute player daily claim behavior for the player command family. |
| `player.daily.status` | operator | cli, paper, velocity, web | Execute player daily status behavior for the player command family. |
| `player.exchange.commit` | operator | cli, paper, velocity, web | Execute player exchange commit behavior for the player command family. |
| `player.exchange.quote` | operator | cli, paper, velocity, web | Execute player exchange quote behavior for the player command family. |
| `player.exchange.rates` | operator | cli, paper, velocity, web | Execute player exchange rates behavior for the player command family. |
| `player.exchange.reconcile` | player | paper | Reconcile an exchange correlation before Paper restores inventory. |
| `player.home.delete` | operator | cli, paper, velocity, web | Execute player home delete behavior for the player command family. |
| `player.home.get` | operator | cli, paper, velocity, web | Execute player home get behavior for the player command family. |
| `player.home.list` | operator | cli, paper, velocity, web | Execute player home list behavior for the player command family. |
| `player.home.set` | operator | cli, paper, velocity, web | Execute player home set behavior for the player command family. |
| `player.inspect` | operator | cli, paper, velocity, web | Execute player inspect behavior for the player command family. |
| `player.kit.claim` | operator | cli, paper, velocity, web | Execute player kit claim behavior for the player command family. |
| `player.kit.list` | operator | cli, paper, velocity, web | Execute player kit list behavior for the player command family. |
| `player.link.begin` | operator | paper | Issue a one-time Discord linking code to a Minecraft player. |
| `player.link.remove` | operator | paper | Remove the Minecraft player account link. |
| `player.load` | operator | cli, paper, velocity, web | Execute player load behavior for the player command family. |
| `player.mail.inbox` | operator | cli, paper, velocity, web | Execute player mail inbox behavior for the player command family. |
| `player.mail.read` | operator | cli, paper, velocity, web | Execute player mail read behavior for the player command family. |
| `player.mail.send` | operator | cli, paper, velocity, web | Execute player mail send behavior for the player command family. |
| `player.moderation.ban` | operator | cli, paper, velocity, web | Execute player moderation ban behavior for the player command family. |
| `player.moderation.mute` | operator | cli, paper, velocity, web | Execute player moderation mute behavior for the player command family. |
| `player.moderation.status` | operator | cli, paper, velocity, web | Execute player moderation status behavior for the player command family. |
| `player.moderation.unban` | operator | cli, paper, velocity, web | Execute player moderation unban behavior for the player command family. |
| `player.moderation.unmute` | operator | cli, paper, velocity, web | Execute player moderation unmute behavior for the player command family. |
| `player.note.create` | operator | cli, paper, velocity, web | Execute player note create behavior for the player command family. |
| `player.note.list` | operator | cli, paper, velocity, web | Execute player note list behavior for the player command family. |
| `player.party.accept` | operator | cli, paper, velocity, web | Execute player party accept behavior for the player command family. |
| `player.party.create` | operator | cli, paper, velocity, web | Execute player party create behavior for the player command family. |
| `player.party.info` | operator | cli, paper, velocity, web | Execute player party info behavior for the player command family. |
| `player.party.invite` | operator | cli, paper, velocity, web | Execute player party invite behavior for the player command family. |
| `player.party.leave` | operator | cli, paper, velocity, web | Execute player party leave behavior for the player command family. |
| `player.points.balance` | operator | cli, paper, velocity, web | Execute player points balance behavior for the player command family. |
| `player.points.top` | operator | cli, paper, velocity, web | Execute player points top behavior for the player command family. |
| `player.random-teleport.complete` | operator | cli, paper, velocity, web | Execute player random-teleport complete behavior for the player command family. |
| `player.random-teleport.history` | operator | cli, paper, velocity, web | Execute player random-teleport history behavior for the player command family. |
| `player.random-teleport.quote` | operator | cli, paper, velocity, web | Execute player random-teleport quote behavior for the player command family. |
| `player.random-teleport.refund` | operator | cli, paper, velocity, web | Execute player random-teleport refund behavior for the player command family. |
| `player.random-teleport.reserve` | operator | cli, paper, velocity, web | Execute player random-teleport reserve behavior for the player command family. |
| `player.recovery.report` | operator | cli, paper, velocity, web | Execute player recovery report behavior for the player command family. |
| `player.report.create` | operator | cli, paper, velocity, web | Execute player report create behavior for the player command family. |
| `player.report.dismiss` | operator | cli, paper, velocity, web | Execute player report dismiss behavior for the player command family. |
| `player.report.list` | operator | cli, paper, velocity, web | Execute player report list behavior for the player command family. |
| `player.report.resolve` | operator | cli, paper, velocity, web | Execute player report resolve behavior for the player command family. |
| `player.restore` | operator | cli, paper, velocity, web | Execute player restore behavior for the player command family. |
| `player.session.join` | operator | cli, paper, velocity, web | Execute player session join behavior for the player command family. |
| `player.session.leave` | operator | cli, paper, velocity, web | Execute player session leave behavior for the player command family. |
| `player.settings.get` | operator | cli, paper, velocity, web | Execute player settings get behavior for the player command family. |
| `player.settings.hud` | operator | cli, paper, velocity, web | Execute player settings hud behavior for the player command family. |
| `player.settings.set` | operator | cli, paper, velocity, web | Execute player settings set behavior for the player command family. |
| `player.settings.toggle` | operator | cli, paper, velocity, web | Execute player settings toggle behavior for the player command family. |
| `player.shop.list` | operator | cli, paper, velocity, web | Execute player shop list behavior for the player command family. |
| `player.shop.purchase` | operator | cli, paper, velocity, web | Execute player shop purchase behavior for the player command family. |
| `player.shop.refund` | operator | paper | Execute player shop refund behavior for the player command family. |
| `player.snapshot` | operator | cli, paper, velocity, web | Execute player snapshot behavior for the player command family. |
| `player.teleport.request` | operator | cli, paper, velocity, web | Execute player teleport request behavior for the player command family. |
| `player.teleport.take` | operator | cli, paper, velocity, web | Execute player teleport take behavior for the player command family. |
| `player.transfer.saved` | operator | cli, paper, velocity, web | Execute player transfer saved behavior for the player command family. |
| `player.vote.list` | operator | cli, paper, velocity, web | Execute player vote list behavior for the player command family. |
| `player.warning.create` | operator | cli, paper, velocity, web | Execute player warning create behavior for the player command family. |
| `player.warning.list` | operator | cli, paper, velocity, web | Execute player warning list behavior for the player command family. |
| `player.warp.get` | operator | cli, paper, velocity, web | Execute player warp get behavior for the player command family. |
| `player.warp.list` | operator | cli, paper, velocity, web | Execute player warp list behavior for the player command family. |
| `player.warp.set` | operator | cli, paper, velocity, web | Execute player warp set behavior for the player command family. |
