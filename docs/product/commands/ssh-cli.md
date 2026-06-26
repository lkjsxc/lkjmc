# SSH CLI

## Purpose

This document defines the target `lkjmc` CLI commands.

## Initial commands

- `lkjmc doctor`
- `lkjmc status`
- `lkjmc config check`
- `lkjmc config reload`
- `lkjmc db migrate`
- `lkjmc db status`
- `lkjmc jar list`
- `lkjmc jar sync --project paper --channel stable`
- `lkjmc jar import --kind modded-custom --name name --path path/to/server.jar`
- `lkjmc shop list`
- `lkjmc shop item upsert ITEM --title-key KEY --price POINTS`
- `lkjmc kit list`
- `lkjmc kit upsert KIT --title-key KEY --reward-points POINTS --cooldown-hours HOURS`
- `lkjmc vote list`
- `lkjmc vote link upsert ID --title-key KEY --url URL`
- `lkjmc vote reward PLAYER_UUID PLAYER_NAME --link ID --points POINTS`
- `lkjmc announcement send --server SERVER --message MESSAGE`
- `lkjmc instance list`
- `lkjmc instance create --id hub --kind paper --template paper-survival`
- `lkjmc instance start hub`
- `lkjmc instance stop hub`
- `lkjmc instance restart hub`
- `lkjmc instance delete hub`
- `lkjmc instance logs hub --lines 120`
- `lkjmc player inspect PLAYER`
- `lkjmc player points-top --limit 10`
- `lkjmc player snapshot PLAYER --name NAME --source INSTANCE --payload PATH`
- `lkjmc player restore PLAYER --snapshot SNAPSHOT_ID`
- `lkjmc moderation reports --limit 20`
- `lkjmc moderation report resolve REPORT_ID`
- `lkjmc moderation report dismiss REPORT_ID`
- `lkjmc moderation warn PLAYER_UUID PLAYER_NAME --reason REASON`
- `lkjmc moderation warnings PLAYER_UUID --limit 20`
- `lkjmc moderation note PLAYER_UUID PLAYER_NAME --body BODY`
- `lkjmc moderation notes PLAYER_UUID --limit 20`
- `lkjmc moderation ban PLAYER_UUID PLAYER_NAME --reason REASON`
- `lkjmc moderation unban PLAYER_NAME`
- `lkjmc moderation mute PLAYER_UUID PLAYER_NAME --reason REASON`
- `lkjmc moderation unmute PLAYER_NAME`
- `lkjmc moderation status PLAYER_UUID`
- `lkjmc audit tail --lines 100`
- `lkjmc verify`

## Current status

The CLI implements doctor, status, config check/reload, database
migration/status, audit tail, moderation report review/close, warning, mute, and
ban/status commands, jar list/import/sync/inspect/prune, shop, kit, and vote
link/reward administration, announcements, player
inspect/points-top/snapshot/restore, `verify`, and the current instance lifecycle/log
commands.
