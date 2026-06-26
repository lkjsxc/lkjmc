# SSH CLI

## Purpose

This document defines the current `lkjmc` CLI command surface.

## Global flags

- `lkjmc [--socket PATH] [--json] ...` selects the Unix socket and compact JSON
  output when supported.

## Runtime and config

- `lkjmc doctor`
- `lkjmc status`
- `lkjmc verify`
- `lkjmc config check [--path PATH]`
- `lkjmc config reload`
- `lkjmc db migrate`
- `lkjmc db status`
- `lkjmc db reset-test` requires `LKJMC_TEST_RESET_DATABASE=1`.
- `lkjmc audit tail [--lines N]`

## Jar and instance operations

- `lkjmc jar list`
- `lkjmc jar inspect QUERY`
- `lkjmc jar import --kind KIND --name NAME --path PATH`
- `lkjmc jar sync --project PROJECT --channel stable [--version VERSION]`
- `lkjmc jar prune --yes`
- `lkjmc instance list`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE [--command CMD]`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE [--jar-asset UUID]`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE [--memory-mb MB]`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE [--server-port PORT]`
- `lkjmc instance start ID`
- `lkjmc instance stop ID`
- `lkjmc instance restart ID`
- `lkjmc instance delete ID --yes [--force]`
- `lkjmc instance logs ID [--lines N]`

## Player and moderation operations

- `lkjmc player inspect PLAYER_UUID`
- `lkjmc player points-top [--limit N]`
- `lkjmc player snapshot PLAYER_UUID NAME SOURCE --payload PATH`
- `lkjmc player restore PLAYER_UUID --snapshot SNAPSHOT_ID`
- `lkjmc moderation reports [--limit N]`
- `lkjmc moderation report resolve REPORT_ID`
- `lkjmc moderation report dismiss REPORT_ID`
- `lkjmc moderation warn PLAYER_UUID PLAYER_NAME --reason REASON`
- `lkjmc moderation warnings PLAYER_UUID [--limit N]`
- `lkjmc moderation note PLAYER_UUID PLAYER_NAME --body BODY`
- `lkjmc moderation notes PLAYER_UUID [--limit N]`
- `lkjmc moderation ban PLAYER_UUID PLAYER_NAME --reason REASON`
- `lkjmc moderation unban PLAYER_NAME`
- `lkjmc moderation mute PLAYER_UUID PLAYER_NAME --reason REASON`
- `lkjmc moderation unmute PLAYER_NAME`
- `lkjmc moderation status PLAYER_UUID`

## Product administration

- `lkjmc shop list`
- `lkjmc shop item upsert ITEM --title-key KEY --price POINTS`
- `lkjmc kit list`
- `lkjmc kit upsert KIT --title-key KEY --reward-points POINTS --cooldown-hours HOURS`
- `lkjmc vote list`
- `lkjmc vote link upsert ID --title-key KEY --url URL`
- `lkjmc vote reward PLAYER_UUID PLAYER_NAME --link ID --points POINTS`
- `lkjmc announcement send --server SERVER --message MESSAGE`

## Source owners

Root parsing lives in `crates/lkjmc-cli/src/args.rs`; family parsers live in
`crates/lkjmc-cli/src/args_*.rs`; execution lives in
`crates/lkjmc-cli/src/commands*.rs`.
