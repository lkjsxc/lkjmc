# E-SYNTHESIS handoff packet 2026-07-12

## Purpose

Map synthesis evidence to possible A-* owner work. This is an evidence packet,
not a controller task: it may not be claimed, transitioned, or used to change
controller state.

## Entry rule

Every future task starts with its owner contract and the hard gate in
[E-SYNTHESIS](../../research/decisions/e-synthesis-20260712.md). `Selected`
means a non-product input for proposal design, never adoption or permission to
merge an experiment harness. `None` is an intentional no-adoption result.

| A-* area | Selected input or rejection | Required owner boundary and exit evidence |
| --- | --- | --- |
| A-CONTRACT | `S-CONTRACT`: CT domain source/shards/checked adapter | Real handler request/response schemas, consumer inventory, stale-output and mutation proof. |
| A-SECURITY | `S-SECURITY`: surface credential/cache/push repair | Fail closed on every uncertainty; off-reactor DB work, revocation-loss/outage/capacity proof. |
| A-EXECUTION | None; `B-E` | Idempotent external effect or explicit scope, crash ordering, cancellation, queue and observation proof. |
| A-DATA | `S-U` only; workflow is blocked | PostgreSQL-only typed/revision/correlation design, retention and atomic workflow faults. |
| A-RUNTIME | None; keyed/lease harnesses rejected | Real daemon ordering, effect/observation recovery, process/database loss and cleanup proof. |
| A-SYNC | None; `B-I` | Attested session, bounded credentials, save/load/transfer/arrival correlation and recovery. |
| A-JVM | None; local models pending | Approved Paper/Folia and Velocity harness, nonblocking scheduler proof, loss/reorder/restart outcome. |
| A-MENU | `S-L`: retain local-only docs | No daemon route or dynamic action until A-JVM exits its gate; client locale/outage/accessibility proof. |
| A-NETWORK | None; compiler rejected | JSON owner contract, desired/observed state, auth, assets, readiness, restart and rollback proof. |
| A-OBS | None; `B-O` | Independent attested observer event, durable operation/request IDs, daemon-path fault and 30 repeats. |
| A-OPS | `S-Q`/`S-U` test inputs | Immutable acquisition, clean Compose, redacted bundle, restore/fault and exact guarded lane evidence. |
| A-PRODUCT | None | Owner journey, consented account/client outcome, degraded recovery, locale/accessibility and maintenance measure. |
| A-QUALITY | `S-Q`: property/state/fault/fuzz/mutation/canary | Pair each technique with a real boundary; no retry masking or replacement of integration proof. |
| A-CUTOVER | `S-U`: private PostgreSQL rehearsal | Versioned migration, backup/restore/rollback, retention, concurrent access and daemon boot proof. |
| A-STATE | None | Update only after a shipped owner task supplies source and deterministic proof; never from research. |

## Compatible combinations

A-CONTRACT may combine with A-QUALITY and A-SECURITY only after real handler
schemas exist. A-DATA/A-CUTOVER may combine with A-QUALITY but not with SQLite,
a broker, or a player transfer claim. A-SECURITY may precede A-JVM/A-SYNC but
does not satisfy attestation. A-MENU remains local-only until A-JVM, A-SYNC,
and A-OBS independently exit their gates. A-EXECUTION, A-RUNTIME, A-NETWORK,
and A-OBS are mutually blocked by the missing effect/correlation evidence;
none may use a fake adapter or synthetic event to proceed.

## Next executable step

The controller may, after its documentation gate, create one owner-scoped
proposal that copies the applicable row's exit evidence into its packet. Until
then retain this packet as documentation-only evidence.
