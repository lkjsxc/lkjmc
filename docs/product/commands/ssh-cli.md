# SSH CLI

## Purpose

This document defines the current `lkjmc` CLI command surface.


## Status

implemented

## Global flags

- `lkjmc [--socket PATH] [--json] ...` selects the Unix socket and compact JSON
  output when supported.

## Admission boundary

The CLI parser retains the catalog grammar, but parser presence is not execution
support. The daemon admits only `status`, `admin role list`, `player settings
get`, `player settings set`, and `player settings hud`. `config reload` returns
non-success `config.restart_required`; every other daemon command below returns
non-success `command.effect_denied` before its handler runs. See the
[command lifecycle](../../architecture/runtime/daemon/command-lifecycle.md).

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
- Planned: `lkjmc observability events [--request ID|--operation ID|--correlation ID] [--limit N]`.
- Planned: `lkjmc support bundle --output PATH`.
- `lkjmc security token plan`
- `lkjmc security token rotate`
- `lkjmc security token status`
- `lkjmc security token verify`

## Bootstrap and network operations

- `lkjmc bootstrap plan --profile playable [--bedrock auto|enabled|disabled] [--java-bind-host HOST] [--java-port PORT] [--java-public-host HOST] [--bedrock-port PORT] [--json]`
- `lkjmc bootstrap apply --profile playable [--bedrock auto|enabled|disabled] [--java-bind-host HOST] [--java-port PORT] [--java-public-host HOST] [--bedrock-port PORT]`
- `lkjmc bootstrap status [--json]`
- `lkjmc bootstrap doctor [--host HOST]`
- `lkjmc network diagnose HOST [--port PORT] [--expect-address ADDRESS] [--direct-address ADDRESS] [--json]`

## Claim operations

- `lkjmc claim list --instance INSTANCE`
- `lkjmc claim delete CLAIM_ID --yes`

## Asset, jar, and instance operations

Current jar operations:

- `lkjmc jar list`
- `lkjmc jar inspect QUERY`
- `lkjmc jar import --kind KIND --name NAME --path PATH`
- `lkjmc jar sync --project PROJECT --channel stable [--minecraft-release RELEASE]`
- `lkjmc jar prune --yes`

Asset operations:

- `lkjmc asset server sync --project paper|folia|velocity [--minecraft-release RELEASE]`
- `lkjmc asset plugin sync --plugin viaversion|viabackwards|geyser|floodgate`
- `lkjmc asset plugin list`
- `lkjmc asset plugin inspect PLUGIN`

Instance operations:

- `lkjmc instance list` shows desired state, observed state, port, presence, and
  autosuspend reason when known.
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE [--command CMD]`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE [--jar-asset UUID]`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE [--memory-mb MB]`
- `lkjmc instance create --id ID --kind KIND --template TEMPLATE [--server-port PORT] [--forwarding-secret-file PATH]`
- EULA-gated create and bootstrap requests omit consent and return
  `adventure.confirmation_required`; only the localized Adventure confirmation
  may originate it.
- Create exits before success when launch source, consent, template, memory, port,
  or duplicate-id checks cannot produce a startable instance.
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

- `lkjmc admin role list`
- `lkjmc admin grant PRINCIPAL ROLE --reason TEXT`
- `lkjmc admin revoke PRINCIPAL ROLE --reason TEXT`
- `lkjmc admin inspect PRINCIPAL`
- `lkjmc admin audit [--lines N]`
- `lkjmc shop list`
- `lkjmc shop seed-defaults`
- `lkjmc shop item upsert ITEM --title-key KEY --price POINTS [--metadata-json JSON]`
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
