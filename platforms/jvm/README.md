# JVM adapters

## Purpose

This directory owns Java 21 common code and the supported Velocity and
Paper/Folia plugin adapters.

## Supported runtime surface

Velocity owns authenticated player identity, `/lkjmc` registration, parsing,
completion, status output, and transfer requests to the fixed `hub` and
`survival` routes. Remote work leaves the event loop immediately and completes
through bounded continuations.

Paper owns `/menu`, `/docs`, the slot-8 menu entrypoint, and a local bundled-doc
browser. Its menu bundle contains exactly five routes: `root` plus four docs
routes. Root contains inert `/lkjmc` command guidance and a docs link. The only
click effects are local navigation, Back, and Close. Paper does not subscribe to
menu/profile/claim/settings snapshots and ships no menu mutation, confirmation,
refresh, generic action body, or daemon command port.

Both platforms own one common runtime and start one dedicated heartbeat reporter
after installation. Each reporter uses its instance-bound, heartbeat-only
credential for empty-body loopback requests under a bounded deadline. JVM child
processes receive no daemon PostgreSQL or bootstrap credential.

## Bindings

The binding generator derives the closed sync wire contract from canonical
daemon transport source and verifies `contracts/sync.json` as its deterministic
JVM projection. It also verifies the canonical command shard manifest and the
closed JVM consumer projection. Generated Java is source-owned and checked in;
Gradle candidates and plugin jars remain ignored build output.

The generated sync transport no longer contains a `menus` domain or menu catalog
payload. Remaining internal sync types are not a supported Paper player surface,
and the installed Paper lifecycle subscribes to none of them. A type or generated
file is not evidence that its domain is supported.

## Lifecycle and scheduling

Lifecycle replacement and close are serialized and bounded. Listener
registrations are removed on close. Scheduler callbacks do not wait on database,
filesystem, network, process, or worker futures. Paper menu interactions are
synchronous local inventory operations; stale route/session/render metadata is
rejected before any navigation effect.

## Verification

`gradlew :platforms:jvm:paper:jvmProbes` exercises scheduler ownership, typed
bindings, routing, transfer outcomes, lifecycle replacement, and jar
containment.

`gradlew :platforms:jvm:paper:menuProbes` runs seven deterministic menu probes:
all five routes, golden frames, navigation without unintended close,
daemon-independent local docs, locale parity, stale-render/session behavior,
and jar absence of the removed remote-snapshot and mutation menu classes.
`menuCheckerMutations` inverts the reduced contract and loader constraints.

These are candidate-jar checks. They do not prove a real login, command,
transfer, or inventory click; those require the guarded live client lane.
