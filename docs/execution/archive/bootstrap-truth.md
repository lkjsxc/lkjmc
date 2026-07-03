# Bootstrap truthfulness

## Purpose

Make playable bootstrap truthful before more product surfaces depend on it.


## Status

completed

## Current status

Implemented for root and migration planning, exhaustive apply/recording,
enabled optional-plugin blocking, and bootstrap status plan diagnostics.

## Scope

- Plan `root.ensure` and `database.migrate` when facts show they are needed.
- Apply every `BootstrapEffect` variant through a real adapter.
- Remove catch-all success paths from apply and step recording.
- Block enabled optional plugins when assets, dependencies, ports, or safety
  checks fail; auto mode may withdraw with diagnostics.
- Include plan outcome, diagnostics, planned effects, instance state, plugin
  state, and public connection text in bootstrap status.

## Owner docs

- `docs/architecture/bootstrap/effects.md`
- `docs/architecture/bootstrap/planner.md`
- `docs/architecture/plugin/third-party-policy.md`
- `docs/architecture/runtime/daemon/status.md`

## Verification

Run `cargo fmt`, targeted core bootstrap tests, daemon bootstrap tests when
present, `./scripts/check-lines.py`, and `./scripts/check-docs.py`.
