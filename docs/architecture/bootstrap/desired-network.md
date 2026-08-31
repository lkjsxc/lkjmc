# Desired network

## Authority

The closed `network` object in the canonical JSON configuration is the only authored fleet intent.
It owns revision, typed instances, routes, listeners, authentication, forwarding, immutable assets,
and runtime capabilities. Bootstrap does not maintain a second topology.

Instance IDs are opaque. Kind, desired state, integration, readiness, listener, assets, and memory
are explicit fields. The fleet is bounded, listener bind hosts are literal IP addresses, listeners
have unique protocol sockets, routes reference existing backend instances, and exactly one configured
Velocity kind selects the Java entrypoint. That invariant does not reserve an ID or port.

## Rendering and persistence

Bootstrap deterministically derives every backend address and the first route's default target for
the selected Velocity instance. It renders all configured backend registrations in stable ID order
and passes their bounded ID list to the Velocity plugin. Adding, removing, renaming, stopping, or
changing a backend within the typed contract requires configuration and durable reconciliation, not
source changes.

Apply stores canonical intent and operation facts in PostgreSQL before external effects. Before a
no-op or mutation it compares configuration, persisted instance and asset facts, rendered files,
verified process identity, and readiness. A difference names the instance and produces explicit
drift; no source silently coerces another.

## Readiness and recovery

Desired-running instances are started and checked by their declared supported oracle. Stopped or
suspended instances remain inactive. A custom or modded server with unsupported readiness can be
configured stopped but cannot be accepted as running.

A failure before all admitted runtime effects may be classified failed. Once a process effect or
unknown commit is possible, the durable attempt remains unknown until a fresh observation adopts or
safely stops the owned process. PID, executable identity, Linux start ticks, and owned process group
are checked; a socket or PID alone is not readiness.

Immutable assets and the forwarding secret are verified independently. EULA consent is not a
bootstrap request field: systemd's Rust pre-start authority validates the host policy and
materializes each required per-instance file before the daemon can start.
