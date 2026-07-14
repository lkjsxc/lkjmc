# Velocity adapter boundary

The production adapter uses the reserved `lkjmc-owned-` registration namespace.
Only names in that namespace are reconciled or removed; all other proxy
registrations are unrelated and untouched. A ready typed routing snapshot must
be current at an exact revision. Reconciliation verifies desired names against
the proxy's actual registrations and reports unavailable on any mismatch.

Connection request completion advances only to `CONNECTED`. Arrival requires a
separate trusted attestation; the production verifier is unavailable until the
daemon exposes that workflow API.
