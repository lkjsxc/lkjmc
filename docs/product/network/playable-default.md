# Playable network contract

## Shape

A supported network has one explicitly configured Velocity instance and a bounded finite collection
of configured backends. IDs, backend count, ports, and route order are operator data. The repository
example uses `edge-gateway` and `quartz-world` only as examples; those strings carry no role.

Velocity is the sole intended public Java listener. PostgreSQL, daemon HTTP, management sockets,
credentials, and backend listeners remain private. Velocity uses online mode, secure key
authentication, and modern player-information forwarding. Backend online mode and the private
forwarding secret are rendered from the same typed configuration.

## Readiness

A desired-running Velocity entrypoint requires its configured status-protocol probe. A
desired-running Paper/Folia/Purpur backend requires verified process identity, fresh plugin
heartbeat, and fresh observed Velocity registration before it is joinable. Stopped or suspended
instances are not required to be ready and remain inactive through update. An active custom/modded
server with no supported oracle fails preflight rather than receiving process-only readiness.

`lkjmc bootstrap status --json` reports the dynamic instance set, desired and observed state,
listener, process health, readiness source/age, registration, and joinability. Truncation or a set
difference is an error at update acceptance.

## Player boundary

Velocity's `/lkjmc status`, completion, and `/lkjmc server <instance-id>` enumerate the
Rust-generated backend inventory. Velocity performs the actual connection request and reports its
future's outcome; a request is not proof that the same player arrived. Real-player acceptance remains
a separate evidence tier.
