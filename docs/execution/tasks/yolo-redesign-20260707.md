# YOLO redesign 2026-07-07

## Purpose

This ledger records completion of `tmp/lkjmc-yolo-redesign-plan-20260707`
against the current repository. Repository state and owner docs remain the
authoritative behavior record.

## Scope

The pass implemented the plan's correctness, contract, runtime/Kubernetes,
security, JVM, operations, and verification slices that can be completed in this
environment. Live external smokes remain guarded and were reported as skipped
when prerequisites were absent.

## Task ledger

| ID | Status | Evidence |
| --- | --- | --- |
| W0-T01 | done | Entry docs read; ledger and blocker pointer created before implementation edits. |
| W0-T02 | done | Baseline and final fast/full/Compose gates run. |
| D19 | done | Migration `036` accepts Kubernetes/runtime observations; DB regression added. |
| D20 | done | `daemon.json.example` matches `LkjmcConfig`; example check wired into fast/full. |
| D21 | done | Japanese ASCII prose translated; locale quality and Java scans strengthened. |
| D22 | done | Folia task registry is concurrent and removes one-shot handles; tests added. |
| D23 | done | Authz uses transport subjects; forged actor/platform body tests added. |
| C01 | done | Command registry schema, schema fields, and strict schema-file checks added. |
| C02 | done | Registry/dispatch/JVM target checks cover new security bindings. |
| C03 | done | Menu schema added and semantic menu check requires it. |
| R01 | done | Runtime lifecycle avoids recording stop success before real stop and state write. |
| R02 | done | Kubernetes planner consumes command, args, env, port, kind, work dir, and readiness. |
| R03 | done | Desired-state docs and stop behavior now preserve honest failed-effect state. |
| S01 | done | New migrations have fresh-schema DB-backed regression coverage. |
| S02 | done | State docs clarify transaction/effect boundaries; stop success path tightened. |
| SEC01 | done | Hashed scoped tokens, create/revoke commands, auth lookup, and CLI support added. |
| SEC02 | done | Web sessions expire/renew, set Max-Age/Secure, logout revokes, CSRF scoped to API. |
| J01 | done | JVM command tree includes scoped-token commands and registry resource test covers them. |
| J02 | done | Menu contract docs/checks cover schema and failure-state semantic validation. |
| OPS01 | done | CI has docs-contracts and Compose lanes plus report artifact upload. |
| OPS02 | done | Real pg_dump/pg_restore and checksum scripts/runbooks added. |
| LIVE01 | done | `verify-live` run truthfully skipped absent guarded prerequisites. |

## Verification ledger

- `./scripts/check-lines.py`: pass.
- `./scripts/check-docs.py`: pass.
- `./scripts/check-command-docs.py`: pass.
- `./scripts/check-locales.py`: pass.
- `./scripts/check-menus.py`: pass.
- `./scripts/check-config-examples.py`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: pass.
- `cargo test --workspace`: pass.
- `./gradlew --no-daemon test`: pass.
- `./scripts/verify-fast.sh`: pass, `skips=db-backed/live-smokes/gradle-shadowJar`.
- `./scripts/verify-full.sh`: pass, `skips=live-smokes`.
- `docker compose --profile verify run --rm verify`: pass, `skips=live-smokes`.
- `./scripts/verify-live.sh`: pass with `ran=none` and all live smoke guards skipped.

## Handoff notes

Live Minecraft, Bedrock, Discord, and Kubernetes smokes still require their guard
variables and external credentials/targets. Skips are not passes.
