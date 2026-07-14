# Bootstrap planner

## Purpose

This contract defines the pure planning model for playable bootstrap.


## Status

implemented

## Inputs

The pure inspector receives the parsed JSON `NetworkConfig`, its canonical
intent digest, durable desired/apply state, and observed filesystem, asset,
port, and runtime facts. It performs no database, filesystem, process, network,
or Kubernetes work.

## Output

Inspection returns deterministic ordered changes, unsupported capability
reasons, and one of `blocked`, `changes`, or `no-op`. Each change names its
instance and exact action. `network_intent::inspect` is the only compiler from
network intent and observation to an inspection plan. The daemon translates
that exact plan to effects for both `bootstrap.plan` and `bootstrap.apply`;
there are no exported compatibility plans, desired-network models, or second
compiler.

## Rules

- Missing EULA acceptance blocks Paper, Folia, and Purpur start effects.
- PostgreSQL absence blocks playable bootstrap.
- `bootstrap.plan` may return a blocked outcome with diagnostics, but
  `bootstrap.apply` returns a daemon error when blocking diagnostics exist.
- Missing root directories plan `root.ensure`.
- Missing schema migrations plan `database.migrate`.
- Missing daemon HTTP token plans secure token-file generation.
- Missing Velocity forwarding secret plans secure secret generation.
- Missing Velocity or Folia server jars plan server asset sync effects.
- Missing `lkjmc` plugin jars plan build and asset registration effects.
- Unverified ViaVersion or ViaBackwards assets are withdrawn in auto mode and
  block in enabled mode.
- Unverified Geyser or Floodgate assets withdraw Bedrock in auto mode and block
  in enabled mode.
- Backend port conflicts allocate from the configured range and update configs.
- Managed instance drift is reconciled idempotently.
- Unmanaged directory conflicts block rather than overwrite.
- Plugin-only changes plan affected restarts without unrelated rewrites.

## Apply boundary

Apply validates all local ports, secrets, assets, ownership, and readiness
inputs before effects. Kubernetes additionally requires mounted configuration,
secrets, and assets; an absent declaration returns `unsupported` before calling
the adapter. A short transaction records desired intent, attempt, and fence,
then releases PostgreSQL before locked filesystem, asset, readiness, or process
work. Apply reacquires a connection only to verify the same fence and append
steps or observations. A pool of size one therefore remains available while an
external effect blocks.

Every apply, including a candidate no-op, invokes the fenced A-RUNTIME observer
and checks rendered files, private secret metadata, immutable asset bytes,
listeners, and process identity. Durable health is history, not current fact.
An absent owned process is drift and plans restart; a stale or unowned identity
is a denial. The resulting observation and repair are appended to history.

## Idempotency

A plan is empty only when canonical desired intent and observed files, assets,
listeners, and processes match. Reapplying that revision records a no-op attempt
without rewriting files or restarting processes. Changes are sorted by instance
id and effect family, independent of JSON array order.
