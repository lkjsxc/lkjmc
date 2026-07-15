# Clean-room lab

## Purpose

Define the reproducible, disposable operations lab and its evidence boundary.

## Status

implemented

## Source boundary

`scripts/run-operations-lab.py` exports the recorded commit with `git archive`
into a private temporary directory. It never copies the caller worktree,
`target/`, Gradle state, jars, credentials, logs, or host build caches. The lab
adds a generated credential canary only as an ignored `.env` file, builds the
pinned Compose images without cache, and scans retained evidence and images for
the canary. A lane fails if the checkout is dirty, the commit is not recorded,
a command is retried, a required effect is skipped, or cleanup leaves an owned
container, network, volume, image, process, database, or partial artifact.

## Evidence schema

The runner writes one private JSON document after all eight required probes pass.
It contains only `schemaVersion`, `commit`, `seed`, `lanes`, and `cleanup`.
Each lane contains `probe`, `status`, `commands`, `skips`, and `artifacts`;
each command contains an argument array and integer exit code. Every lane
executes its named boundary; mutation checks supplement rather than replace the
real effect. Required lanes cannot skip. Artifact records contain a relative
path and SHA-256. Container
outputs are returned to the private lab-directory owner; raw output is bounded,
redacted before writing, and scanned again before publication. The exact probes
are:

- `clean-clone-compose`;
- `restore-boot-pass`;
- `installer-rerun-pass`;
- `artifact-provenance-pass`;
- `toolchain-acquisition-pass`;
- `verification-evidence-pass`;
- `fault-lab-pass`;
- `ci-compose-retained`.

## Execution

Run `scripts/run-operations-lab.py --output /tmp/a-ops-evidence.json`. The host
needs Git, Docker with Compose, and enough private storage for two fresh builds.
The runner creates PostgreSQL only through its uniquely named Compose project,
runs the full Compose gate twice, and removes only resources carrying that
project name. Rootless, cluster, signing, player, and public-network outcomes
remain external prerequisites and are recorded as explicit skips outside the
eight required lanes.

A lab pass proves only that commit and host. It does not prove production
capacity, trusted signing identity, external routing, or player recovery.
