# E-CONTRACT contract-source experiment

## Purpose

Test three isolated contract-source candidates against current status, instance
lifecycle, profile-transfer, and shop-purchase handlers without changing product
contracts or registering an adapter.

## Status

experiment

## Baseline

`contracts/commands.json` gives all strict commands the permissive generic request
schema. The four selected commands name `cli` and `web`, but the current
withdrawal boundary excludes Java daemon and Discord command adapters.

## Hypothesis

Bounded request definitions plus deterministic command and locale shards reject
payload drift before a checked applicable consumer. A language-neutral source
may also generate compileable Rust and Java declarations, but cannot override
F-SAFETY withdrawal or prove a consumer absent from current sources.

## Slice and invariants

- Commands: `status`, `instance.start`, `player.transfer.saved`, and
  `player.shop.purchase`.
- Read actual registry, daemon registration, CLI, web, menu, JVM, and Discord
  sources; do not start a daemon, Minecraft runtime, or external service.
- Retain F-SAFETY: no Java daemon client, dynamic menu, Discord command, token,
  or placeholder adapter is created.
- The probe must distinguish current generic-schema acceptance from candidate
  unknown-field and wrong-type rejection.
- A menu catalog result may prove only repository validation; it must not claim
  Java menu consumption or a daemon-backed menu.

## Variants and combinations

- Hand-authored field maps with deterministic command/locale shards.
- A standalone Rust typed-descriptor emitter with the same shard/check adapter.
- A language-neutral JSON source generating transient Rust and Java declarations.
- Run each source with shard and checked-consumer inspection, then run the
  strongest source with error/identity/effect metadata and with menu compilation
  boundary inspection.

## Workload

The harness uses fixed valid, unknown-field, and wrong-type request bodies; seed
`20260711`; three deterministic generation repetitions; and a deliberate stale
output mutation. It invokes `rustc` and `javac` only for isolated generated
artifacts and records unavailable toolchains as blocked.

## Measurements

Correctness is pass/fail rejection and exact source coverage, not latency.
Record generated hashes, emitted file counts, compiler exits, source-line
consumer matches, and the current menu/JVM/Discord boundary.

## Paths and evidence

Base commit: `d20e5e5`. Allowed writes are `docs/research/**`. The isolated
[harness](e-contract-20260711.py), [source fixture](e-contract-20260711-source.json),
and [Rust descriptor](e-contract-20260711-rust.rs) are not product code. The
[run](../runs/e-contract-20260711.md) and
[decision](../decisions/e-contract-20260711.md) retain the result. Raw output
stays below a uniquely named `/tmp` root.
