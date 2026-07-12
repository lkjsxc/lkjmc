# E-SECURITY credential experiment hypothesis

## Purpose

Define bounded, fail-closed credential candidates before any adoption work.

## ID and catalog evidence

`E-SECURITY` tests `SE-SURFACE-CRED`, `SE-AUTH-CACHE`, `SE-AUTH-PUSH`,
`SE-SHORT-CRED`, `SE-UNIX-PEER`, `SE-WEB-SESSION`, and `SE-RATE-LIMIT` from the
[security catalog](../ideas/security.md). The baseline transport performs a
synchronous database lookup for each scoped bearer request.

## Hypothesis and real slice

Three isolated, test-only candidates authenticate a database-backed,
surface-bound credential without recording a raw credential: direct PostgreSQL
lookup; a bounded decision cache; and that cache with a PostgreSQL revision
notification. A short-lived HMAC-signed surface credential is compared against
the direct lookup under the same command policy. The slice uses a migrated,
throwaway PostgreSQL schema, the actual `daemon_tokens` lookup/revocation API,
the real command-policy function, a Docker different-UID Unix socket peer, and
an actual `LISTEN` connection loss/reconnect.

## Invariants

No candidate may authorize a forged actor, principal, surface, permission, or
body field; bypass expiry or revocation; print a raw credential; render a root
token; or alter shared controller or production authorization behavior. A
cached result must be invalidated by its credential revision and expire at a
bounded deadline. On notification disconnect, reconnect-before-repair, or a
missed repair deadline, the test candidate must deny cache reads. Database work
must stay outside the async reactor candidate. Unix-peer authorization remains
independent of bearer credentials.

## Variants and combinations

Run direct lookup, bounded cache, cache plus revision invalidation, and signed
short-lived credentials. Combine the cache-plus-invalidation candidate with the
existing command policy. Run revocation, expiry, forged fields, database delay,
restart, positive/negative clock skew, web-session expiry/logout, and bounded
rate-pressure cases. Drop the actual listener before revocation, notify while it
is absent, reconnect it, and use a periodic revision read to repair. Test Unix
identity using a host listener and a disposable Docker `--user 65534:65534`
client through a read-only bind mount.

## Workload and measurements

Use process-generated credentials only in memory, 32 requests after warmup per
candidate, a fixed public seed label `e-security-20260711`, a 200 ms cache TTL,
a 250 ms signed lifetime, and a 100 ms induced database delay. The loss test
uses a 50 ms repair deadline. Record median and maximum elapsed time,
allowed/denied counts, repair delay, reactor ticker progress, observed peer UID,
and cleanup result. Keep raw test output only in ignored `tmp/` paths.

## Base, worktree, and allowed writes

Base: `d20e5e532db9d3a5577f567dd6a5a24fdc51eea1`; worktree:
`/tmp/pi-agent-3f095c15-d84b-4d1-71a5edb6`; immutable evidence tip:
`48fe62545b917011e1974e49555aec7c4ad65b58`. That isolated source changed
candidate harness modules under `crates/**` plus `docs/research/**`; the
modules are intentionally excluded from this evidence-only tree. Owner documents:
`docs/architecture/runtime/daemon/commands/security.md`,
`docs/architecture/web/security.md`, and `docs/product/discord/security.md`.

## External prerequisite

The real PostgreSQL run requires `LKJMC_STORE_TEST_DATABASE_URL` pointing to a
throwaway database. The harness creates an isolated local Docker PostgreSQL
container and records its exact outcome and rerun command if unavailable. The
Unix proof requires Linux, local Docker access, and a cached `python:3.12-slim`
image; `--pull=never`, `--network=none`, `--read-only`, dropped capabilities,
and `no-new-privileges` prevent a test client from fetching or retaining state.
