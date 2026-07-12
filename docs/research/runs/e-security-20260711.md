# E-SECURITY credential experiment run

## Purpose

Record a reproducible, secret-safe comparison of test-only credential candidates
without changing product authentication or controller state.

## Scope

Task: `E-SECURITY`; base: `d20e5e532db9d3a5577f567dd6a5a24fdc51eea1`;
initial probe: `5a567e1aa9cf44a7d459eaf8854c9267ac1cd44e`; correction probe:
`f0c975bccf7a06c5c1ea21a96b22a60cb86bacf4`; immutable evidence tip:
`48fe62545b917011e1974e49555aec7c4ad65b58`; public seed label:
`e-security-20260711`. Its candidate Rust modules are absent here, so every
command below is source-tip evidence. The harness generated credential material only in
memory, stored only its hash, and did not render a root token, bearer value,
cookie, signing key, or database password.

Host: Linux 7.0.0-27-generic, 20 CPUs, 32,768 MiB memory; Rust 1.96.0, Cargo
1.96.0, Docker 29.5.3, and Docker Compose 5.1.4. The database was a fresh,
removed `postgres:16-alpine` container bound to loopback with a generated
password. It contained only the test migration schema.

## Reproduction and raw evidence

Replay only in a detached worktree of the immutable evidence tip:

```sh
git worktree add --detach /tmp/lkjmc-e-security-evidence \
  48fe62545b917011e1974e49555aec7c4ad65b58
REPO=/tmp/lkjmc-e-security-evidence
sh "$REPO/docs/research/runs/e-security-20260711.sh"
```

It retains sanitized command and test logs below
`$REPO/tmp/e-security-20260711/` until explicit cleanup. `umask 077` protects
the raw root. The command ledger records the database URL only as `<redacted>`.
A successful replay uses the same command with a different owned raw root:

```sh
E_SECURITY_ARTIFACT_ROOT="$REPO/tmp/e-security-20260711-replay" \
  sh "$REPO/docs/research/runs/e-security-20260711.sh"
```

Raw evidence from the source run is `commands.log` (`result: passed`),
`docker-peer-test.log`, `database-candidates.log`, and `reactor-test.log`. The
peer log's raw assertion line was:

```text
E_SECURITY_PEER client=docker_user_65534 observed_uid=65534 socket_owner_uid=1001 denied=true dir_mode=711 socket_mode=602 mount_readonly=true cleanup=true
```

The database and reactor raw metric lines were respectively:

```text
E_SECURITY_METRIC direct_median_us=107085 direct_max_us=118671 cache_median_us=0 cache_max_us=7 rate_allowed=8 repair_us=1209
E_SECURITY_METRIC worker_ticks=22
```

A post-run search of that owned raw root for `postgres://`,
`POSTGRES_PASSWORD`, and `password=` had no matches.

## Source-tip commands and results

| Command | Exit | Result |
| --- | ---: | --- |
| `sh docs/research/runs/e-security-20260711.sh` | 0 | Started and removed isolated PostgreSQL; all listed probes passed. |
| `cargo test -p lkjmc-daemon e_security -- --nocapture` | 0 | Signed, same-UID Unix-peer, and web-session tests passed; database tests awaited the isolated URL. |
| `cargo test -p lkjmc-daemon e_security_unix_peer_different_uid_docker -- --ignored --nocapture` | 0 | Docker `--user=65534:65534` connected to the bound host socket; kernel credentials reported UID 65534 and the different-UID policy denied it. |
| `LKJMC_STORE_TEST_DATABASE_URL=<redacted> cargo test -p lkjmc-daemon e_security_credential_candidates -- --ignored --nocapture` | 0 | Direct, cache, normal notification, notification loss, reconnect, repair, policy, expiry, restart, and rate tests passed. |
| `LKJMC_STORE_TEST_DATABASE_URL=<redacted> cargo test -p lkjmc-daemon e_security_reactor_no_block -- --ignored --nocapture` | 0 | Worker-path lookup passed while the ticker advanced 22 times. |

The 32-request database workload induced `pg_sleep(0.1)` before every direct
lookup. Its direct median was 107,085 us and maximum 118,671 us; cached median
was 0 us and maximum 7 us. The loss/reconnect periodic revision read completed
in 1,209 us. Under 32 attempts, the gate allowed 8 database lookups and denied
the remaining 24 before lookup. The separate replay reported 106,692 us median,
157,999 us maximum, 12 us cache maximum, 1,408 us repair, and 25 ticker
intervals; its peer evidence matched the main run exactly.

## Named probe outcomes

| Probe | Outcome | Evidence |
| --- | --- | --- |
| `credential-candidates-run` | pass | Isolated database candidate command and metrics above. |
| `forgery-negative-suite` | pass | Database policy and signed-forgery assertions passed. |
| `revocation-bounds` | pass | Direct denial, normal notification receipt, and repair invalidation passed. |
| `notification-loss-repair` | pass | Dropped listener denied cache reads; reconnect had no old notification; revision repair invalidated. |
| `reactor-no-block` | pass | Worker-path command passed with 22 ticker intervals. |
| `unix-peer-proof` | pass | Actual Docker different-UID client produced observed kernel UID 65534 and denial. |
| `security-combinations-run` | pass | Cache, policy, session, and rate tests passed. |

## Fault outcomes

- The direct record was surface-bound to Paper and a menu scope. Forged
  principal, actor, body surface, and body permission fields were denied by the
  real command policy; an altered signed credential was denied.
- Revocation denied a direct lookup. A live listener received a normal revision
  notification. The test then dropped that listener, revoked and notified while
  absent, and connected a new listener that received no old notification.
  Between disconnect and an explicit periodic revision repair, the guard denied
  cache reads; repair saw revision 2, invalidated the record, and the 50 ms
  guard deadline independently denied after its repair window elapsed.
- An expired database row was denied. A signed claim was denied six milliseconds
  before issuance and one millisecond after its 250-millisecond expiry. It still
  verified after database revocation before expiry, so signed-only credentials
  do not meet revocation requirements.
- The worker lookup left 22 five-millisecond ticker opportunities runnable. A
  preliminary direct synchronous PostgreSQL attempt inside Tokio panicked with
  `Cannot start a runtime from within a runtime`; it was not retained as a
  candidate path.
- The Docker client used `--pull=never`, `--network=none`, `--read-only`,
  `--cap-drop=ALL`, and `no-new-privileges`; its socket-directory bind mount was
  read-only. The host created a UUID directory at mode 0711 and socket at 0602,
  so the numeric unprivileged client could connect without directory listing or
  write access to the mount. `peer_cred` observed UID 65534 while socket owner
  UID was 1001; the actual different-UID policy decision was deny.

## Cleanup and containment

The database trap executes `docker rm -f "$container"` on `EXIT`, `HUP`,
`INT`, and `TERM`. The peer client has Docker `--rm`; after its process joins,
the test drops both socket endpoints, removes the socket and its UUID directory,
and asserts the directory is absent. To remove inspected raw evidence only,
run this exact command after confirming `REPO`:

```sh
rm -rf -- "$REPO/tmp/e-security-20260711"
rm -rf -- "$REPO/tmp/e-security-20260711-replay"
```

An initial direct store-create attempt exited 101 with `error serializing
parameter 6`; the experiment inserted its disposable row with parameter-free
SQL and continued to exercise the actual lookup and revocation APIs. This is a
baseline defect, not a workaround for product behavior. No controller,
migration, listener, command registration, production auth path, or state
matrix changed.
