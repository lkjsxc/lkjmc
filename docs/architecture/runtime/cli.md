# CLI

## Purpose

This document defines the implemented SSH-friendly operator surface.


## Status

implemented

## Implemented families

`crates/lkjmc-cli/src/args.rs` parses these top-level families:

- `lkjmc doctor`
- `lkjmc status`
- `lkjmc verify`
- `lkjmc network diagnose ...`
- `lkjmc config ...`
- `lkjmc db ...`
- `lkjmc audit ...`
- `lkjmc jar ...`
- `lkjmc instance ...`
- `lkjmc player ...`
- `lkjmc moderation ...`
- `lkjmc shop ...`
- `lkjmc kit ...`
- `lkjmc vote ...`
- `lkjmc announcement ...`

`doctor`, `status`, `config reload`, `audit tail`, jar operations, player
operations, moderation operations, shop, kit, vote, announcement, and instance
operations use HTTP `POST /command` over the daemon Unix socket. Database
migration, status, and guarded test reset use `LKJMC_DATABASE_URL` directly.
`verify` runs the repository verification script in the current checkout.

## Source owners

- Root parser: `crates/lkjmc-cli/src/args.rs`.
- Family parsers: `crates/lkjmc-cli/src/args_*.rs`.
- Root dispatcher: `crates/lkjmc-cli/src/commands.rs`.
- Family handlers: `crates/lkjmc-cli/src/commands_*.rs`.
- Product command contract: [../../product/commands/ssh-cli.md](../../product/commands/ssh-cli.md).

## Output

`lkjmc status` prints daemon uptime, database state, counts, roots, HTTP, and
reconciler state for humans. `lkjmc network diagnose HOST` prints DNS, SRV, TCP,
status ping, comparison, and next-action details. `--json` emits compact
machine-readable JSON for commands that return daemon or local data. Human
output must be truthful and should not hide failures behind success text.
