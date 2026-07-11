# Final acceptance

## Purpose

This file records the final redesign-bundle acceptance disposition.

## Status

implemented

## Defect register disposition

- D1 fixed by `963a347`: migration tests derive expected migrations.
- D2 fixed by `436dfba` and `1294a46`: claim writes are atomic and first-contact
  identity writes are FK-safe.
- D3 fixed by transactional multi-write sweeps in `436dfba`, `1294a46`, and later
  handler work; remaining record-only paths are documented.
- D4 fixed by `243da10` and `8816c85`: CI runs the Compose verify profile.
- D5 and D6 fixed by `d58a018`: inbound daemon and Discord HTTP use axum.
- D7 fixed by `4318169`: daemon runtime uses a PostgreSQL pool.
- D8 fixed by `def2245`: menu rows send real save-first transfers.
- D9 fixed by `1eab388` plus this acceptance slice: linking, unlink, and linked
  Discord wake delegate daemon commands.
- D10 fixed by `008600d` and `8816c85`: fast/full/live tiers name skips.
- D11 fixed by `d1e26da` and `9a0693b`: status docs and split state files.
- D12 fixed by `d58a018` and `21ea232`: marker modules and no-op semantics were
  removed or documented.
- D13 fixed by `21ea232`: the insert-only outbox table is dropped.
- D14 fixed by `4e6761a` and `9323fb1`: locales are single-source and Gson-parsed.
- D15 re-scoped (historical): this plan pass kept Java admin dispatch explicit
  and command-registry checked. Later containment withdrew Java admin dispatch
  and no current Java command-registry coupling remains.
- D16 fixed by `8816c85`: Compose uses named profiles.
- D17 fixed by `8816c85`: smoke harnesses and config examples are populated.
- D18 fixed by `21ea232`: recovery reports are documented record-only reports.

## Acceptance annotations

Deterministic gates are green: fast verification, DB-backed cargo tests, Gradle,
and Compose full verification. The status checker was intentionally violated and
failed, then the change was reverted.

The 2026-07-07 YOLO plan pass added D19-D30 coverage for Kubernetes observed
state persistence, config examples, Japanese locale quality, Folia task
lifecycle, transport-subject authorization, command schemas and historical menu
catalog checks, Kubernetes launch inputs, scoped token metadata, web sessions,
CI lanes, and backup/release runbooks. Later containment withdrew the menu
schema and daemon menu semantics.

Live smokes were not run in this final pass because this environment does not
provide EULA-approved playable runtime credentials, live Discord credentials,
Kubernetes access, or public Bedrock/Minecraft endpoints. `./scripts/verify-live.sh`
reported `ran=none` and skipped each guard variable.

## Design divergences

- The Dockerfile uses cacheable toolchain, Rust dependency, Gradle dependency,
  verify, and playable stages; no pre-redesign timing baseline was available.
- Historical: Java admin dispatch extraction was re-scoped in this plan pass.
  Later containment withdrew that Java dispatch surface entirely.
