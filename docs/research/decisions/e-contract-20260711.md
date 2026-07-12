# E-CONTRACT source decision

## Purpose

Interpret the isolated contract-source experiment without selecting, merging, or
registering a product contract implementation.

## Status

experiment

## Disposition

combine

The permissive generic request shape is unsuitable for typed coverage. Keep
the language-neutral source, domain shards, and checked adapters as a tentative
combination for E-SYNTHESIS review only. This is not adoption.

## Evidence

- Baseline and run: [E-CONTRACT run](../runs/e-contract-20260711.md).
- Hypothesis and candidates: [E-CONTRACT experiment](../experiments/e-contract-20260711.md),
  `e-contract-20260711-source.json`, and `e-contract-20260711-rust.rs`.
- Candidate commit: `a6dcf8eef4cc539ff6b50ecab6a5b2fea77ce2d6` in the isolated worktree.
- Independent review: pending; no outcome is promoted without it.

## Comparison

| Candidate | Drift faults | Shards and checked adapter | Binding evidence | Decision role |
| --- | --- | --- | --- | --- |
| Hand-authored maps | rejects | deterministic | no generated binding | reference only |
| Rust descriptor | rejects | deterministic | rustc test/emitter | rejected as source of truth: descriptor repeats fields |
| Neutral source | rejects | deterministic | generated Rust test and Java 21 compile | tentative combination input |

The generic request schema accepts unknown fields and wrong field types. Each
candidate rejected both across status, instance start, profile-transfer saved,
and shop purchase bodies. The neutral generator detected an altered output and
three shard builds had the same `be00a664…0316` hash.

## Invariants and boundaries

The daemon registry test proved 137 registry names map to exactly 137 registered
handlers. It does not prove per-handler payload validation. `check-menus.py`
validated four local static docs routes, but `LocalDocsMenu` reads the bundled
documentation resource rather than `contracts/menus`; menu compilation therefore
has no daemon-menu or Java-consumption claim.

The surface probe found CLI and web literals for `status` and `instance.start`,
but neither consumer maps `player.transfer.saved` or `player.shop.purchase`.
Java daemon mappings remain absent and Discord's payload is empty. JVM
containment and four Discord withdrawal tests passed. No adapter was added, so
F-SAFETY withdrawal remains intact.

## Reason and costs

The neutral source was the only candidate that produced both compileable language
artifacts while preserving deterministic shards and fault rejection. It still
models only four bounded experiment bodies, not actual handler types or
responses. Adding it now would duplicate the generic registry and falsely imply
that missing public consumers or withdrawn Java/Discord paths work. The Rust
variant demonstrates the same drift protection but retains a second descriptor
beside its types. Hand maps are simplest but duplicate cross-language fields.

## Reconsideration and follow-up

E-SYNTHESIS must retain all three candidate commits and decide whether an
adoption task first creates strict schemas from real handlers, checked consumer
inventories, and a stale-output gate. It must separately resolve the absent CLI
and web mappings or correct their catalog surface declarations. Java and Discord
may not be reconsidered until F-SAFETY trusted identity/session attestation
permits real adapters. No adoption or cleanup task is authorized by this
experiment; no candidate is prepared for merge.
