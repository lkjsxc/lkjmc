# Documentation refresh

## Purpose

This task keeps the repository contract truthful and easy for future agents to
scan before they edit code.

## Contract

- Root status, current-state, blockers, and task docs must agree on shipped
  behavior.
- Runtime command docs must list daemon and CLI surfaces from source owners.
- Minecraft command docs must match registered Paper and Velocity commands.
- Permission docs must match Java constants and plugin metadata.
- Locale docs must state that English and Japanese keys ship together.

## Verification

Run these before considering this task complete:

```sh
./scripts/check-lines.py
./scripts/check-docs.py
```

Contract drift scripts belong in `./scripts/verify.sh` once added.
