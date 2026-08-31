# Playable network operations

## Current shape

The supported configuration describes one private PostgreSQL database and Rust daemon, one selected
Velocity entrypoint, and a bounded collection of private backends. `edge-gateway`,
`quartz-world`, and any other IDs in examples are not reserved. Listener ports and fallback order
come from typed configuration.

A typed Rust first-install command exists for a prepared systemd container, but clean installation
remains unsupported until fresh supported-host acceptance is observed. Existing deployments consume
only exact immutable release bytes through [the Rust operations authority](../install.md); do not
treat local implementation or Docker checks as a supported-host installation.

## Restart reconciliation

The canonical systemd unit runs the daemon from `/opt/lkjmc/releases/current`. Root
`ExecStartPre` calls `lkjmc-ops fence check` and `lkjmc-ops eula materialize`;
`ExecStartPost` first runs `lkjmc --json bootstrap apply` as the service user, then calls
`lkjmc-ops bootstrap after-start`. The former reconciles the typed desired fleet after every
daemon start; the latter verifies the private daemon boundary and selected Velocity listener. No
service hook invokes Python, a shell helper, or a fixed instance list.

The post-start command waits for the private daemon boundary, validates the exact build and
PostgreSQL status, compares the full configured and persisted instance sets, probes the designated
Velocity listener, and requires readiness only for desired-running instances. A stopped instance is
not a startup failure.

## Inspection

```sh
runuser -u lkjmc -- /opt/lkjmc/releases/current/bin/lkjmc --json status
runuser -u lkjmc -- /opt/lkjmc/releases/current/bin/lkjmc --json bootstrap plan
systemctl show lkjmc-daemon.service -p ActiveState -p SubState -p NRestarts -p MainPID -p Result
```

A converged plan is an exact no-op only when typed configuration, PostgreSQL, generated state,
artifacts, process observations, and required readiness agree. The command never writes an EULA
policy; that root-owned acceptance record is a separate explicit operator action.

## Evidence boundary

A successful systemd start plus protocol/plugin observations is process and disposable or
supported-host evidence only for the named environment. It is not a real-player login, completion,
command, transfer, menu, or production observation.
