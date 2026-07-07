# Verification state

## Purpose

This file records shipped verification behavior.

## Status

implemented

## Current behavior

- Documentation topology, line-limit, bootstrap-doc, asset-doc, command-doc,
  menu-contract, permission-doc, config-schema, and locale catalog checks are
  implemented.
- `./scripts/verify-fast.sh`, `./scripts/verify-full.sh`, `./scripts/verify.sh`,
  and `./scripts/verify-live.sh` provide named verification tiers with explicit
  skip summaries. `verify.sh` delegates to the full tier.
- Dockerfile stages, Compose `verify`/`playable`/`discord` profiles, and
  `tests/smoke/` protocol harnesses are implemented.
- Daemon and opt-in smokes cover claim dispatch, live Paper claim behavior,
  protocol-level break/place protection, and the command/menu smoke harness when
  prerequisites are set. The engine-backed playable command/menu smoke passed
  with EULA acceptance in this pass.
- Installer, playable Compose, and live Minecraft smokes opt in because they need
  privileged host changes, Docker, EULA acceptance, or network downloads.
- The CI workflow has a docs/contracts lane and a cached Compose verify lane;
  the Compose lane runs `docker compose --profile verify run --rm verify`.
- The latest unguarded `./scripts/verify-live.sh` run reported `ran=none` and
  skipped Minecraft, claim, playable, Bedrock, Discord, and Kubernetes smokes
  because their guard variables were not set.

## Verification status

The default gates are `./scripts/verify-fast.sh` and
`docker compose --profile verify run --rm verify`. `check-menus.py` validates
menu documents, locale keys, command references, reachability, and generated
route-doc parity in those tiers. Live/playable smokes are opt-in and must be
reported as skipped unless their guard variables are set.
