# Clean-room lab

## Purpose

Define the reproducible, disposable operations lab and its evidence boundary.

## Status

implemented

## Source boundary

`scripts/run-operations-lab.py` exports the recorded commit with `git archive`
into a private temporary directory. It never copies the caller worktree,
`target/`, Gradle state, jars, credentials, logs, or host build caches. The lab
adds generated random credential canaries only in excluded inputs, builds the
pinned Compose images without cache, and scans the complete exported context,
release closure, saved image layers, and retained evidence for both canaries
and credential values. The saved unique-project verifier image must be a regular
tar no larger than 2 GiB. Its bounded Docker manifest and configs must declare
every retained file; canonical regular members, config/layer digest agreement,
and every distinct layer's compressed and expanded content are verified once.
Shared layer references are valid. Missing, unreferenced, conflicting duplicate,
traversing, symlink, device, oversized, or over-count members fail before the
recursive credential scan. Nested image-layer links are never materialized;
authored archive links still fail. Literal printf placeholders in authored URL
templates, documented tool example values, and diagnostic prose after an
authentication keyword are not credential values. A lane fails
if the checkout is dirty, the commit is
not recorded, a command is retried, a required effect is skipped, or cleanup leaves an owned
container, network, volume, image, process, database, or partial artifact.

## Evidence schema

The runner writes one private JSON document after all eight required probes pass.
It contains only `schemaVersion`, `commit`, `seed`, `lanes`, and `cleanup`.
Each lane contains `probe`, `status`, `commands`, `skips`, and `artifacts`;
each command contains an argument array and integer exit code. Every lane
executes its named boundary; mutation checks supplement rather than replace the
real effect. Required lanes cannot skip. Artifact records contain a relative path and SHA-256. Their exact set equals
all retained regular files below `raw/`; an unindexed retained file, duplicate
index path, changed file, oversized log, or file outside that bounded root
fails finalization. The final evidence JSON is outside that closure to avoid
self-reference. Container outputs are returned to the private lab-directory owner with
owner-only directory and regular-file modes. The restore drill hands off every
retained raw artifact, including negative-case inputs and daemon logs; it may
instead retain a bounded sanitized copy and remove the raw source. Three
old-style unreadable raw roots are executable rejection fixtures. Raw output is
bounded, redacted before writing, and scanned again before publication.

All evidence and secret-tree traversal uses one deterministic descriptor-relative
walker. It opens directories and regular files with no-follow flags, compares
each opened descriptor identity with its no-follow directory-entry stat, and
fails on a race, symlink, special file, root/device crossing, unreadable entry,
non-private mode, or count, byte, or depth overflow. Nothing is omitted. The
unreadable-directory probe runs with dropped privileges so a `000` ordinary
directory containing a hidden canary is rejected even on a root host. The exact probes
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
The runner creates PostgreSQL only through uniquely named Compose projects,
runs the full Compose gate twice, creates an independent-like second Git export,
and removes only resources carrying those project names. It runs all eight
semantic probes and their delete, extra, duplicate, traversal, checksum, symlink, special-file,
unreadable-entry, race, root-crossing, permission, count, byte, and depth
mutations. Installer no-op/rollback, acquisition-checksum, and upload-gating
mutations also run. Rootless, cluster, signing, player, and public-network outcomes
remain external prerequisites and are recorded as explicit skips outside the
eight required lanes.

A lab pass proves only that commit and host. It does not prove production
capacity, trusted signing identity, external routing, or player recovery.
