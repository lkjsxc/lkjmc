# Clean-room verification

## Current boundary

The maintained clean-room boundary is the pinned verifier image used by the required workflow. It
receives an exact committed source bundle, not the caller's `target/`, Gradle cache, credentials,
logs, worlds, or ambient release outputs. PostgreSQL is supplied by a unique project-scoped
container with no published port.

The former local operations-lab runner and fixed-topology Docker recovery implementation are
deleted. Their accepted deterministic semantics are Rust-owned tests; any future live systemd and
Minecraft matrix must consume a new exact Rust-authority release and a noncanonical typed fleet.

## Evidence

A valid clean-room run records the exact commit, pinned image identities, command exits, skipped
external boundaries, PostgreSQL result, release manifests, reproducibility comparison, secret scan,
and cleanup result. Every retained file is private, bounded, indexed by path/size/SHA-256, and
traversed without following links. A missing required database probe, unreadable entry, extra file,
secret canary, failed cleanup, or nonzero command remains a failure.

This proves only deterministic/container/PostgreSQL/artifact boundaries for that exact revision.
It does not prove supported-host installation, real systemd ownership, Minecraft readiness,
protocol-client behavior, a real player, public exposure, or production.
