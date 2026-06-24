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
- `lkjmc instance list`
- `lkjmc instance create --id hub --kind paper --template paper-survival`
- `lkjmc instance start hub`
- `lkjmc instance stop hub`
- `lkjmc instance restart hub`
- `lkjmc instance delete hub`
- `lkjmc instance logs hub --lines 120`
- `lkjmc player inspect PLAYER`
- `lkjmc player snapshot PLAYER --name NAME --source INSTANCE --payload PATH`
- `lkjmc player restore PLAYER --snapshot SNAPSHOT_ID`
- `lkjmc audit tail --lines 100`
- `lkjmc verify`

## Current status

The CLI implements doctor, status, config check/reload, database
migration/status, audit tail, jar list/import/sync/inspect/prune, player
inspect/snapshot, and the current instance lifecycle/log commands. Player
restore and verify command are not implemented yet.
