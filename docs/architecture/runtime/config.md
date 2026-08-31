# Runtime configuration

## Authority

`/etc/lkjmc/lkjmc.json` is the canonical typed semantic input. `lkjmc-core` parses it once for
all Rust consumers; `lkjmc-ops` reuses those types and validation instead of maintaining a
privileged schema copy. Unknown members, unsafe paths, duplicate IDs or sockets, dangling routes,
unreferenced assets, placeholder digests, invalid bounds, and conflicting kind/integration/readiness
contracts fail before effects.

PostgreSQL stores durable desired and operation facts. Rendered instance files and observed process
or readiness state are derived boundaries, not alternative configuration authorities. Every field is
restart-required; `config.reload` reports `config.restart_required` and does not partially adopt
new input.

## Fleet

`network.instances` contains 1–64 instances. An instance ID is opaque lowercase kebab-case and
does not imply kind or role. Each instance explicitly declares:

- `kind`: `velocity`, `paper`, `folia`, `purpur`, `vanilla-custom`, or
  `modded-custom`;
- `desiredState`: a retained typed lifecycle state;
- `integration`: `velocity`, `paper-compatible`, or `none`;
- `readiness`: `velocity-status`, `plugin-heartbeat`, or `unsupported`;
- one listener, memory bound, and immutable asset IDs.

The current single-network contract requires exactly one Velocity kind but gives its ID no special
value. Paper, Folia, and Purpur require the Paper-compatible integration and plugin heartbeat.
Custom/modded kinds currently require no plugin and unsupported readiness, so they may remain
stopped but cannot be described as service-ready.

Listeners own protocol, literal-IP bind address, port, and explicit public hosts; IPv6 sockets are
rendered with brackets. Routes own target and ordered fallback IDs. One public Java boundary may
select the Velocity listener; backend listeners are private in the supported production direction.
Renderer ordering is deterministic and derives the default target from the first configured route.

## Assets, plugins, and credentials

Network assets bind absolute paths to non-placeholder SHA-256 identities. Instance kinds and
integration determine which release jar, scoped heartbeat credential, EULA file, and readiness
oracle apply. Names do not select those behaviors. Required files are independently checked before
start or update.

The generated Velocity launch environment contains the configured backend IDs, while each Java
process receives only its instance ID/kind, server port, heartbeat endpoint, and instance-bound
credential path. The daemon clears its inherited secret environment before spawn.

## Database and private HTTP

`database.poolSize` defaults to 8 and is bounded to 1–64. Database password and daemon HTTP bearer
tokens are read from private files and are never printed. Enabled daemon HTTP accepts only a literal
loopback socket. The forwarding secret path is absolute and private.

## Runtime adapters

`runtime.adapter` is explicit. `local-process` is the supported process owner. Kubernetes input is
validated but remains an unsupported production path where required fencing or process guarantees
are unavailable. No adapter is selected from an instance name.

## Verification

`scripts/check-config-examples.py` invokes the production Rust parser; it is not a second schema.
`contracts/config/README.json` and its shards map every accepted field to a Rust source owner.
Rust tests cover two noncanonical fleets, duplicate and drift failures, readiness contracts, plugin,
credential, EULA, listener, and route derivation.
