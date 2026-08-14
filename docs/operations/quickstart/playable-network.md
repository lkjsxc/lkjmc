# Playable network operations

## Current production topology

The supported deployed shape is one local PostgreSQL database and Rust daemon,
one online-mode Velocity proxy on TCP `25591`, and private Folia backends
`hub` (`127.0.0.1:25566`) and `survival` (`127.0.0.1:25567`). The daemon HTTP
listener (`127.0.0.1:8765`) and PostgreSQL remain private.

There is no clean-host playable quickstart yet. The old checkout installer is
withdrawn. Existing deployments use the immutable update command documented in
[host deployment](host-install.md).

## Restart reconciliation

The canonical systemd unit starts the daemon from
`/opt/lkjmc/releases/current`. Its packaged `ExecStartPost` helper waits for the
private Unix socket, retries only the bounded stale-identity adoption case, and
then invokes the daemon's typed `bootstrap plan`/`bootstrap apply` path. No
service script launches Java or renders instance state independently.

The helper passes the bootstrap EULA admission flag only when an existing
acceptance record is present. It never writes or fabricates a marker. Missing
acceptance, server assets, scoped heartbeat credentials, readiness, or exact
process ownership fails the systemd start.

## Inspection

```sh
runuser -u lkjmc -- /opt/lkjmc/releases/current/bin/lkjmc --json status
runuser -u lkjmc -- /opt/lkjmc/releases/current/bin/lkjmc \
  --json bootstrap plan --profile playable --bedrock disabled
systemctl show lkjmc-daemon.service \
  -p ActiveState -p SubState -p NRestarts -p MainPID -p Result
```

A converged plan is an exact no-op. Hub and survival must be process-healthy,
ready, proxy-registered, and joinable from fresh plugin heartbeats. Velocity is
process-healthy but correctly reports `not-a-backend` rather than backend
readiness.

## Truthfulness boundary

Systemd success, protocol pings, registration, logs, and rendered menus prove
installation and network readiness only. `/lkjmc` parsing/completion/status,
successful and failed transfers, `/menu`, and `/docs` remain unaccepted until an
authorized online-mode client performs them.
