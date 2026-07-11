# Server lifecycle

## Purpose

This contract defines visible server create, start, registration, and join
states.

## Status

implemented

Implemented: Velocity registration reports, stored registration TTL, joinability
reducer, connect-host derivation, actionable create planning, explicit start
controls, and bounded log surfacing.

## Workflow

Server create, start, and join are one workflow:

1. preflight prerequisites;
2. create durable instance config;
3. allocate a port and render files;
4. install required jar and plugin assets;
5. start the runtime;
6. wait for readiness heartbeat;
7. register with Velocity when required;
8. report joinability truthfully;
9. transfer players only when ready and registered.

## Create planning

`instance.create.plan` returns `startable`, structured missing prerequisites,
recommended actions, jar asset candidates, plugin asset status, port plan,
runtime adapter, and whether proxy registration is desired. An EULA-gated plan
with absent or false consent instead returns the bodyless, non-retryable
`adventure.confirmation_required` response before database planning. Menus show
a real fix action when the daemon supports one, such as jar or plugin sync.
Otherwise they show a precise operator hint.

Create-and-start or `startAfterCreate` requires confirmation because it allocates
and starts durable resources. A stopped created instance is visible but not
joinable. Start failures store bounded runtime observations and logs.

## Joinability

A server is joinable only when it is running, healthy, heartbeat-ready, has a
connect host and port, and is registered with Velocity when proxy registration is
desired. Public lists keep stopped, starting, failed, suspended, hidden, and
not-registered servers visible with exact disabled reasons.

Velocity reports actual managed-server registration state, connect host, port,
and registration failures to the daemon with a short TTL. `instance.list`
returns registration desired, registered state, age, connect address, health,
heartbeat, joinable flag, and join-disabled reason.

## Connect host

Local-process backends on the same host may use loopback. Docker Compose uses
service or container network names. Kubernetes uses service DNS or configured
connect addresses. Product code must not hardcode `127.0.0.1` for every
backend.

## Verification

Core tests cover the joinability reducer. Daemon and store tests cover
registration reports, TTL, and `instance.list`. Velocity tests cover reporting.
Playable smoke creates, starts, waits for registration, and joins a ready server.
