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

The example declares Velocity `proxy` at Java TCP `0.0.0.0:25565` and Folia
`hub` at loopback TCP `25566`; the default route targets `hub`. Velocity is
online and modern forwarding is mandatory. The forwarding secret is generated
once in its absolute private file, reused, and never returned in output.

## Durable ownership

Apply stores canonical intent, its authored revision, request correlation, and
a monotonic database revision in PostgreSQL before any external effect. Apply
status and append-only attempts record planned, applying, observed, failed, or
unsupported outcomes without secret bytes. A later apply inspects durable and
runtime facts and repairs a partial result rather than replaying stale effects.

## Assets and optional features

Required server and plugin assets must match their declared SHA-256 before
installation. Paper and Purpur are Paper-compatible; Folia remains a distinct
scheduler target. Optional Java compatibility or Bedrock assets can be omitted
or withdrawn, but a required missing asset blocks before process launch.
