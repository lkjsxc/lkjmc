# Velocity plugin

## Authority

Velocity owns authenticated proxy player identity, proxy command registration, the live server
registration boundary, and actual connection requests. The Rust renderer supplies the canonical
backend IDs and addresses; Java does not infer a role from an ID.

## Maintained behavior

At startup the plugin parses the bounded `LKJMC_BACKEND_IDS` generated from typed configuration,
rejects empty, duplicate, invalid, or oversized inventories, captures every named live registration
and address in Velocity, installs `/lkjmc`, and only then starts its scoped heartbeat. Each later
heartbeat compares that captured set with the live proxy registry and reports registered, missing,
or route-mismatch observations; the daemon independently checks the complete set against persisted
fleet addresses. The inventory may contain one or many arbitrarily named backends.

`/lkjmc status` asynchronously pings each configured backend. `/lkjmc server <instance-id>`
offers those IDs through Brigadier and uses Velocity's connection-request API. Status work has a
three-second deadline and eight-request bound; transfers have a five-second deadline and
32-request bound. Timeout feedback does not release admission until the platform future settles.
Success, already-connected, in-progress, cancelled, disconnected, timeout, missing registration,
and invalid target remain distinct.

Player identity comes only from Velocity's `Player`. The command does not mutate daemon desired
state and never reports arrival before the connection future completes. Backend registration or
connection-request success is not a real-player observation.

## Lifecycle

Event listeners and the command are registered once per runtime and removed on replacement or
shutdown. Late callbacks are suppressed after close. The heartbeat uses the Velocity instance's
single-purpose credential and starts only after command and inventory checks pass. Focused Java tests
cover noncanonical inventories, completion, status, every transfer outcome, bounded pending work,
shutdown, and repeated lifecycle replacement.

The routing adapter may reconcile lkjmc-owned dynamic registrations from a current typed routing
snapshot without touching unrelated proxy registrations. Cached snapshots and unattested workflow
requests are not player or routing authority.
