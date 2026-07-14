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

`contracts/sync.json` is the JVM-owned closed sync binding contract. The binding
generator also reads the repository command shard manifest and every listed
canonical command shard. `contracts/consumption.json` is the closed JVM command
consumer set. It is empty while daemon command workflow APIs are absent.
Generated Java is deterministic, source-owned, and checked in; Gradle candidates
and plugin jars remain ignored build output. Contract objects and shard listings
are closed: malformed input, an unlisted command shard, a JVM surface in a
canonical shard not represented by the consumer set, or stale generated output
fails `verifyJvmBindings`.

Platform adapters consume generated typed records. Generic JSON parsing is
confined to the common transport codec and never crosses into Paper or Velocity.

## Workflow and effects

Common owns immutable revisioned workflow views. Transitions require exact
identity fields; exact duplicates are stable, while stale, reordered, skipped,
or mismatched events are denied. Terminal success requires an acknowledgement
or observation transition, never request submission.

Each plugin owns exactly one common runtime: one daemon sync coordinator and one
bounded effect executor. Effects use bounded queues, attempts, futures, and
timeouts. Scheduler callbacks only submit work or execute platform API calls;
they never wait on database, filesystem, network, process, or worker futures.
Disable closes admission, cancels work, and performs a bounded off-scheduler
join. Repeated lifecycle cycles leave no workers.

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

The probe task exercises real adapter classes through disposable Paper/Folia
scheduler and Velocity proxy fakes, repeats loss/reorder/restart/cancellation,
and inspects real plugin jars. Queue saturation must reject without blocking;
shutdown must complete queued and active results and leave no worker. Setting
`-PjvmProbe=<name>` runs one named probe, while the default runs all eight.
Gradle `check` depends on binding verification and the relevant harnesses.
Mutation tests invert freshness, identity, acknowledgement, ownership, and
owned-registration conditions and must fail.
