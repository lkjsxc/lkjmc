# Desired network

## Purpose

This contract defines the desired default network consumed by the
bootstrap planner.


## Status

implemented

## Selected contract

`network` in the main JSON configuration is the only authored network intent.
Its closed shape contains `revision`, `instances`, `routes`, `listeners`,
`auth`, `forwarding`, `assets`, and `capabilities`. The Rust production parser
rejects unknown members, duplicate ids, empty topology, invalid ports, unsafe
secret paths, unreferenced routes or assets, duplicate listeners, and values
outside documented count and memory bounds.

Each instance names one owner (`lkjmc-daemon`), implementation, listener,
asset, memory bound, and desired state. Routes name a listener, target, and
ordered fallback targets. Listeners own protocol, bind address, port, and public
hosts. Assets name immutable SHA-256 content. Capabilities select
`local-process` or `kubernetes` and explicitly declare mounted config, secret,
and asset support.

## Default topology

The repository example declares Velocity `proxy` at Java TCP `127.0.0.1:25565`
and Folia `hub` at loopback TCP `25566`; the default route targets `hub`. The
single-host target additionally declares Folia `survival` at loopback TCP
`25567` as the ordered fallback. Bootstrap deterministically renders every
non-Velocity instance into Velocity's server map, with `hub` first. Velocity is
online and modern forwarding is mandatory. The forwarding secret is generated
once in its absolute private file, reused, and never returned in output.

Example and installer configuration intentionally contain no server or plugin
artifacts because the repository has not acquired immutable coordinates for
those inputs. Their instance asset lists are empty and `mountedAssets` is false,
so inspection and apply deny the network before filesystem, process, or
bootstrap effects. Operators must supply acquired files with independently
verified, non-placeholder SHA-256 values before enabling that capability.

## Durable ownership

Apply stores canonical intent, its authored revision, request correlation, and
a monotonic database revision in PostgreSQL before any external effect. Apply
status and append-only attempts record planned, applying, observed, failed,
unknown, no-op, or unsupported outcomes without secret bytes.

A failure known to precede every runtime effect can finish as `failed`; rendered
files are not described as rolled back. Once a runtime effect is admitted, a
timeout, lost database commit, or daemon loss leaves that attempt `unknown`.
Neither a stale instance row nor a new request may clear that uncertainty.

Before every apply or no-op decision, bootstrap calls the A-RUNTIME observer
and inspects managed files, private secret permissions, immutable asset bytes,
the exact configured database/config jar binding, listeners, and runtime
identity. A binding mismatch is drift and forces a fenced stop before
replacement; a healthy process cannot turn the wrong jar into a no-op. The
local adapter adopts only a child matching
its fenced identity marker: PID, executable device/inode, and Linux start ticks.
A missing owned child is drift and is restarted. A stale marker or unowned
process denies apply rather than being replaced. Interrupted attempts are then
adopted or stopped according to current network intent. The old attempt retains
`unknown` plus the real observation, and a new correlated attempt records the
inspection and repaired result. No path claims rollback.

## Recovery fault matrix

| Fault boundary | Durable old outcome | Required recovery proof |
| --- | --- | --- |
| before effect | `failed` | no child and queryable attempt |
| after config render | `failed` | rendered drift is inspected; no process rollback claim |
| after child start | `unknown` | marker-fenced child is observed and adopted or stopped |
| after observation, before attempt commit | `unknown` | adapter observation is repeated before success |
| daemon restart | unchanged until recovery | restarted adapter identifies the same child; no orphan |

Every case must leave no unowned child, no false success, and queryable old and
retry attempts. Observation or reconciliation failure stays non-success and
blocks a retry effect.

## Assets and optional features

Required server and plugin assets must match their declared SHA-256 before
installation. Uniform, repeated, and known placeholder digests are invalid.
Paper and Purpur are Paper-compatible; Folia remains a distinct scheduler
target. Optional Java compatibility or Bedrock assets can be omitted or
withdrawn, but every running server requires acquired immutable assets and a
missing asset capability blocks before any effect.
