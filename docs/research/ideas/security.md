# Security research ideas

## Purpose

Imported identity and secret-operation candidates; none expands access today.

## Catalog evidence

Source: supplied `experiments/catalog/security.md`. Each remains untested.

## Candidates

- `SE-SURFACE-CRED` surface-scoped credentials; `SE-PRINCIPAL-ATTEST` adapter proof.
- `SE-ROOT-RETIRE` bootstrap-root confinement; `SE-AUTH-CACHE` bounded revocation.
- `SE-AUTH-PUSH` cache invalidation; `SE-SHORT-CRED` signed-vs-lookup credentials.
- `SE-UNIX-PEER` peer identity; `SE-WEB-SESSION` browser-session controls.
- `SE-RATE-LIMIT` early subject/route limits; `SE-SECRET-PROVIDER` no-print secrets.
- `SE-SUPPORT-REDACT` canary-tested bundles; `SE-AUDIT-DENIAL` safe denial audit.
- `SE-DEPENDENCY-POLICY` provenance checks; `SE-THREAT-TEST` negative suites.

## Decision boundary

Compare credentials under revocation, latency, restart, and clock skew. Test Unix
peer identity separately. Never place usable credentials or externally reachable
control surfaces in a research run.
