# CLI

## Purpose

This document defines the implemented SSH-friendly operator surface.


## Status

implemented

## Implemented families

`crates/lkjmc-cli/src/args.rs` parses these top-level families:

- `lkjmc version`
- `lkjmc doctor`
- `lkjmc status`
- `lkjmc verify`
- `lkjmc network diagnose ...`
- `lkjmc config ...`
- `lkjmc db ...`
- `lkjmc audit ...`
- `lkjmc bootstrap ...`
- `lkjmc jar ...`
- `lkjmc instance ...`
- `lkjmc player ...`
- `lkjmc moderation ...`
- `lkjmc shop ...`
- `lkjmc kit ...`
- `lkjmc vote ...`

Catalog commands use HTTP `POST /command` over the daemon Unix socket, but
catalog parsing is not admission. `status`, `admin role list`, the three
player-settings operations, and the four bootstrap observations/apply commands
can run; `config reload` is restart-required and all other daemon commands
return non-success before their handlers. Network apply additionally requires a
kernel-authenticated local Unix peer.
Database migration, local database status, and guarded test reset use
`LKJMC_DATABASE_URL` directly. `verify` runs the repository verification script
in the current checkout.

## Source owners

- Root parser: `crates/lkjmc-cli/src/args.rs`.
- Family parsers: `crates/lkjmc-cli/src/args_*.rs`.
- Root dispatcher: `crates/lkjmc-cli/src/commands.rs`.
- Family handlers: `crates/lkjmc-cli/src/commands_*.rs`.
- Product command contract: [../../product/commands/ssh-cli.md](../../product/commands/ssh-cli.md).

## Output

`lkjmc status` prints daemon uptime, database state, counts, roots, HTTP,
reconciler state, and the fail-closed lifecycle boundary for humans. Bootstrap
plan, status, and doctor use the ordinary eight-second bound. Local
`bootstrap apply` has one explicit 20-minute bound for immutable-asset verification,
daemon-owned process effects, and readiness; the separate Rust systemd pre-start
boundary must already have verified the host EULA policy and per-instance files. TCP
credentials cannot receive that budget or start the effect.
`lkjmc network diagnose HOST` prints DNS, SRV, TCP, status ping, comparison, and
next-action details; it is local CLI work, not a daemon effect. `--json` emits
compact machine-readable JSON for commands that return daemon or local data.
Human output must be truthful and should not hide failures behind success text.
