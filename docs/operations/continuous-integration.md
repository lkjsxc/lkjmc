# Continuous integration

## Required workflow

`.github/workflows/verify.yml` runs on pushes to `main` and pull requests. `docs-contracts`
runs the fast owner checks. `verify-compose` exports the exact commit, builds the pinned verifier,
starts a unique project-scoped PostgreSQL service, and runs `scripts/verify-full.sh`.

The full verifier runs Rust formatting, workspace Clippy and tests, affected Java tests and shaded
jars, generated-owner checks, migration/data/runtime/network/security/fault probes, the Rust
operations contract, release-inventory mutation checks, and artifact identity checks. With the
Compose database configured, required PostgreSQL probes may not silently skip. Live Minecraft,
supported-host systemd, protocol client, real player, and production remain separate lanes.

The historical fixed-topology recovery lab is deleted. Its no-op, lock, backup, fence, permit,
publication interruption, rollback, changed-ledger, EULA, and recovery semantics now live in focused
Rust tests. CI does not label those deterministic tests as systemd or Minecraft proof.

## Exact release producer

After the verifier passes, the workflow attaches the single advertised source-bundle ref and builds
the exact release twice in separate fresh roots inside the same pinned image. It compares complete
path, type, mode, size, and SHA-256 closure. It then:

1. independently verifies the strict nine-member Rust/Java/declarative inventory;
2. packs the first equal root twice as deterministic POSIX `ustar` and compares both handoffs;
3. safely extracts and rechecks the manifest and embedded Rust/JVM identities;
4. audits the saved verifier image and redacted bounded evidence;
5. secret-scans source context, release, archive, image layers, and retained evidence; and
6. uploads only after every gate and project cleanup succeeds.

The release artifact name binds commit, workflow run, and attempt. It contains only the archive,
archive checksum sidecar, and `release-handoff.json`, with 30-day retention. No tag, GitHub Release,
signature, permanent channel, or installation is implied.

## Independent consumer

`verify-release-artifact` downloads the exact same-run artifact by ID, retrieves and recomputes the
outer artifact-service ZIP digest, verifies metadata/run/commit/expiry, selects Java 21 only to
execute embedded identity checks, safely consumes the inner archive without rebuilding, scans the
download and receipt, and retains the receipt separately. Consumer success is
`release artifact verified`, not installed, running, ready, player-accessible, or production
evidence.

## Failure and cleanup

No retry or `continue-on-error` promotes a failure. The unique Compose project is always taken down
with its volumes and local image; remaining labeled resources fail cleanup. Rejected secret-bearing
or malformed evidence is not uploaded. Bounded diagnostic output preserves the original command exit
and does not replace it.

Local reproduction requires the pinned container toolchain and a fresh PostgreSQL boundary. Host
native checks are useful development evidence but are not equivalent to the workflow's exact
toolchain or retained artifact.
