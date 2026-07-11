# Operations research ideas

## Purpose

Imported reproducibility and recovery candidates; none is an operational promise.

## Catalog evidence

Source: supplied `experiments/catalog/operations.md`. Each remains untested.

## Candidates

- `OP-CLEAN-COMPOSE` clean Docker verification; `OP-ROOTLESS-LAB` least privilege.
- `OP-KUBE-LAB` ephemeral cluster; `OP-SYSTEMD-HARDEN` service sandboxing.
- `OP-RESTORE-DRILL` restore/boot; `OP-FAULT-LAB` injected failures.
- `OP-SUPPORT-BUNDLE` redacted diagnosis; `OP-CAPACITY` supported envelopes.
- `OP-ARTIFACT-MANIFEST` commit-tied inputs; `OP-SBOM` component inventories.
- `OP-SIGN` optional signatures; `OP-DOWNLOAD-LOCK` source/hash metadata.
- `OP-CI-LANES` quick plus Compose; `OP-REPRO-BUILD` repeated hashes.
- `OP-TOOLCHAIN-PIN` verified tools; `OP-SKIP-EVIDENCE` truthful nested records.
- `OP-CUTOVER` rollback drills; `OP-INCIDENTS` fault-tested runbooks.

## Decision boundary

Run restores with selected schemas, workflows, and artifacts; run faults under
load. Rootless, systemd, and Kubernetes evidence need host/cluster prerequisites
recorded as external pending when unavailable.
