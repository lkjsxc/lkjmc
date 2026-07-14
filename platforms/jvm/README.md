# JVM adapter owner contract

## Purpose

This directory owns typed Java 21 bindings, pure workflow decisions, bounded
effect execution, and Paper/Folia and Velocity lifecycle adapters.

## Attestation gate

Trusted live player/session acknowledgement and daemon workflow transition APIs
are absent. Production adapters therefore expose profile application, delivery
acknowledgement, transfer arrival, and authority decisions as unavailable and
perform no player mutation unless a future verifier supplies an exact attested
operation, session, player, profile revision, lease fence, and correlation.
Submitting a save or connection request is never success or arrival.

Disposable scheduler and proxy fakes may exercise real adapter classes. They
are bounded integration harnesses, not live Minecraft, daemon, player, or
arrival proof. External live Minecraft remains a later guarded lane.

## Bindings

The binding generator derives the closed sync wire contract from the canonical
daemon transport source and verifies `contracts/sync.json` as its deterministic
JVM projection. It also reads the repository command shard manifest and every
listed canonical command shard. `contracts/consumption.json` is the closed JVM
command consumer set. It is empty while daemon command workflow APIs are absent.
Generated Java is source-owned and checked in; Gradle candidates and plugin jars
remain ignored build output. Malformed input, source/projection drift, an
unlisted command shard, an unconsumed JVM surface, or stale output fails
`verifyJvmBindings`.

One common closed decoder maps every snapshot and feed result variant to
generated records with domain-specific generated payloads. Every field keeps
its exact JSON kind; in particular, revision and cursor strings never coerce to
numbers. Unknown, missing, out-of-range, fractional, negative, or wrongly typed
fields reject the whole response and advance no cache, required revision, or
cursor. A seven-domain malformed-type matrix checks this boundary. Platform
adapters consume only those generated records; generic JSON never crosses the
common transport codec.

## Menu engine

A source-owned compiled bundle contains all 62 indexed menu routes. Common owns
closed loader, route, dependency, action, view, session, and failure types.
Paper has one inventory adapter for root, dynamic, confirmation, and curated
documentation routes. Menu, permission, claim, and settings views consume the
generated revisioned records; stale and unavailable states fail visibly.
Mutation requires a current capability and exact attestation. No generic daemon
action/body or mutation port is shipped, so no menu click claims mutation
success.

## Workflow and effects

Common owns immutable revisioned workflow views. Transitions require exact
identity fields. A bounded immutable replay history keeps every retained exact
prior signal stable as `DUPLICATE` after later transitions; changed, expired,
stale, reordered, skipped, or mismatched events are denied. Terminal success
requires an acknowledgement or observation transition, never request submission.

Each plugin owns exactly one common runtime: one daemon sync coordinator and one
bounded effect executor. Effects use bounded queues, attempts, futures, and
timeouts. Scheduler callbacks only submit work or execute platform API calls;
they never wait on database, filesystem, network, process, or worker futures.
Lifecycle replacement and close are serialized; replacement completes the prior
bounded off-scheduler shutdown before installing a runtime. Listener
registrations are explicitly removed on close. The real Paper plugin lifecycle
adapter dispatches listener installation through a scheduler-owned stage. Paper
and Velocity harnesses each run 100 enable, disable, and replacement cycles,
allow at most one runtime and one listener set, await the prior close before
replacement, and require zero ownership after disable.

Paper/Folia ownership hops are explicit main/global, entity, and region stages.
Profile and inventory changes use Bukkit APIs on an ownership stage only.
Permission and claim snapshots are hints unless current and exactly revisioned;
uncertainty denies. Java object deserialization is forbidden.

Velocity reconciles only registrations it owns, checks desired against actual,
and leaves unrelated registrations untouched. A real connection completion may
advance a transfer to connected; only a separately trusted arrival observation
may advance it to arrived.

## Verification

`gradlew :platforms:jvm:paper:jvmProbes` runs exactly these probes:

1. `scheduler-blocks-zero`
2. `typed-bindings-all`
3. `folia-ownership-pass`
4. `velocity-routing-pass`
5. `transfer-outcomes-pass`
6. `workflow-ack-pass`
7. `plugin-shutdown-pass`
8. `duplicate-jvm-paths-absent`

`gradlew :platforms:jvm:paper:menuProbes` also runs the exact seven menu probes
documented by the GUI owner. Its disposable protocol-like harness drives the
production adapter but is not a live server or client. `menuCheckerMutations`
inverts loader, freshness, capability, and attestation conditions.

The probe tasks exercise real adapter classes, repeat bounded failure sequences,
and inspect real jars. Setting `-PjvmProbe=<name>` or `-PmenuProbe=<name>` runs
one named probe. Gradle `check` depends on binding, menu-bundle, mutation, and
both probe suites.
