# Verification

## Purpose

This document defines verification tiers and guarded smokes.


## Status

implemented

## Tiers

| Tier | Command | Scope | Final line |
| --- | --- | --- | --- |
| Fast | `./scripts/verify-fast.sh` | docs, contract checks, fmt, clippy, Rust tests without external services | `ok verify-fast skips=...` |
| Full | `./scripts/verify-full.sh` | fast scope plus DB-backed tests when configured, daemon/process/jar/claim checks, installer, plugin/web checks, Gradle test and `shadowJar` | `ok verify-full ran=... skipped=...` |
| Default | `./scripts/verify.sh` | wrapper for the full tier used by agent prompts and local handoff checks | `ok verify-full ran=... skipped=...` |
| Live | `./scripts/verify-live.sh` | opt-in smokes that need external credentials, EULA, Docker networking, or cluster access | `ok verify-live ran=... skipped=...` |

The full summary names each executed deterministic smoke and every unattempted
nested probe with its exact prerequisite. It never collapses a nested skip into
a successful smoke: missing database configuration lists both checksum and
deadline probes. Compose enables the database-backed claim and web smokes; a
local run without their prerequisites reports them as skips.

Compose full verification uses the consolidated profile:

```sh
docker compose --profile verify run --rm verify
```

The safe-operations probe builds a disposable Docker context containing nested
secret-shaped names and proves none reach a scratch image. No Docker executable
or daemon is an exact `docker` skip; an available Docker context failure fails.

## Opt-in smoke guards

| Smoke | Guard |
| --- | --- |
| Minecraft protocol smoke | `LKJMC_MINECRAFT_SMOKE=1` |
| Minecraft claim protocol smoke | `LKJMC_MINECRAFT_CLAIM_SMOKE=1` |
| Playable Velocity/Paper network | `LKJMC_PLAYABLE_SMOKE=1` and `LKJMC_ACCEPT_MINECRAFT_EULA=1` |
| Bedrock/Geyser smoke | `LKJMC_BEDROCK_SMOKE=1`, enabled endpoint, and supported client |
| Discord live smoke | `LKJMC_DISCORD_SMOKE=1`, JSON config, credentials, and interaction access |
| Kubernetes live smoke | `LKJMC_KUBERNETES_SMOKE=1`, config, database URL, `kubectl`, credentials, and disposable namespace |

`verify-live.sh` dispatches only the six guards above. Its summary lists
unguarded lanes as skipped; it cannot prove an external prerequisite exists.
Each guarded script validates additional prerequisites and may fail instead of
skip. Preserve redacted command output and the relevant external observation for
a live proof. Skipped checks must not be reported as passed.

## Truth probes

`./scripts/verify-truth-probes.sh` is the dedicated deterministic
expected-failure harness. It derives reopened IDs from the forensic
prior-acceptance map, requires one nonempty future task and probe for every ID,
and rejects extra, omitted, or duplicate mapping items. It passes only when that
mapping is valid, the known current weak shapes reject, and conforming-fixture
mutations reject.

The six packet selectors are `prior-items-have-probes`,
`old-runtime-shape-rejected`, `generic-schema-rejected`,
`reactor-blocking-detected`, `contracts-size-detected`, and
`probe-mutation-tests`; each is selectable with `--probe`. Detailed selectors
separate missing payload consumers, menu goldens, stale source paths, and the
restore drill. Normal mode fails on the current runtime-wide mutex, generic
schemas, absent consumers and goldens, stale or narrowly scanned source paths,
and missing restore proof. The current command and web dispatch boundaries use
`spawn_blocking`, so the reactor selector is compliant today; its source-boundary
mutation still rejects.

Expected mode is not product, live, or adoption proof. Adoption work must make a
repaired normal probe mandatory rather than treating expected failures as a
green verification tier.

## Laboratory boundaries

`python3 tests/lab/lab_probes.py --probe NAME` is bounded verification
infrastructure, not product evidence. It creates unique roots, schema names,
ports, and Compose projects; it tears down effects and retains only redacted,
bounded artifacts. Artifact redaction must remove registered secrets, sensitive
structured values, URI userinfo for every scheme (including whitespace before
`@`), and each sensitive URL query value through `&`, a fragment, or the line
boundary, including whitespace. `PASS` is an observed boundary. Only an absent
PostgreSQL URL is `SKIP`; every explicit target is parsed and validated before
tool availability, and an unconfirmed or unsafe target is `BLOCKED`.

| Probe | Real boundary and hard requirement |
| --- | --- |
| `postgres-real` | An absent URL skips; an explicit URL is blocked unless it is confirmed with `LKJMC_LAB_POSTGRES_DISPOSABLE=1` and names a loopback `lkjmc_lab_*` database. `psql` is then required. |
| `daemon-http-real` | Locally built daemon over both loopback TCP and a Unix socket. |
| `process-real` | A local child process held alive until laboratory cleanup. |
| `isolation-cleanup` | Held TCP and Unix listeners plus a child process; teardown must release both addresses, remove the socket, and stop the child. |
| `secret-redaction` | Artifact output redacts structured JSON secret values, sensitive keys, every URI credential and sensitive query value, plus Bearer and Basic headers without printing values. |

`LKJMC_LAB_COMPOSE=1` may start only the unique laboratory Compose project;
`LKJMC_LAB_PROTOCOL=1` additionally requires EULA acceptance, Docker, and Java.
These opt-ins and the PostgreSQL target are disposable only. Never target a
production database, Compose project, or player endpoint.

## Test-only fault harness

`./scripts/check-fault-harness.py --probe NAME` selects commit, process,
observation, HTTP deadline, credential lookup, shutdown, and seeded replay
checks. The fixed clock and deterministic schedule never sleep
to create ordering. Pre-commit faults leave a test transaction uncommitted;
after-commit faults record the durable test effect before returning the fault.

The `deterministic-seed-replay` selector arms an effect-boundary `Failpoints`
control in `ScenarioRunner`, observes its `Err`, and compares its bounded
transcript/state with a second run of the same seed. It also verifies a selected
different seed is distinguishable. The versioned
[seed-failure evidence](../execution/fault-seed-replay.json) is readable JSON
for an independent quality reviewer: rerun the stated selector and compare its
printed failure. Its review status is pending, not acceptance.

Rust controls compile only under `#[cfg(test)]`. They are not configuration,
commands, runtime adapters, or plugin registrations. The release selector
rebuilds the daemon and rejects test failpoint markers in its artifact. This harness falsifies test
boundaries; it does not replace database, child-process, or Minecraft proof.

## Store and CLI gates

Store integration tests create per-test PostgreSQL schemas named
`lkjmc_test_<random>` and run migrations inside each schema so parallel test
threads do not share tables. CLI parsing has a Rust unit suite covering global
flags, command-family parsing, and usage failures.

## Playable gate

```sh
LKJMC_PLAYABLE_SMOKE=1 LKJMC_ACCEPT_MINECRAFT_EULA=1 \
  ./scripts/check-playable-smoke.sh
```

The playable target joins through Velocity and may assert the local-safe Paper
`/menu` and `/docs` entrypoints, hotbar/docs UI, and Velocity MOTD/tab-list.
It does not assert `/lkjmc`, Java daemon auth, dynamic daemon menus, or daemon
mutations: those Java adapters are withdrawn pending trusted identity/session
attestation. The smoke owns generated Compose volumes and removes them before
and after the run.
