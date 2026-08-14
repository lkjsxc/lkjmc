# Player synchronization

## Status

not shipped

The current Paper and Velocity lifecycles do not save, load, apply, or transfer
durable player profiles. Historical generated sync records and deterministic
transport tests remain in common code, but no installed platform adapter
subscribes to them and they are not a supported player journey.

[Revisioned transport](revisioned-transport.md) and
[transfer safety](transfer-safety.md) describe internal proof boundaries only.
They do not establish a live Java save, load, application, session, or arrival.
