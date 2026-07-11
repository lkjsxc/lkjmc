# Clean-room lab

## Purpose

This planned runbook defines the isolation and evidence boundary for an operator
clean-room recovery lab.

## Status

planned

## Isolation boundary

Use disposable compute, a fresh PostgreSQL database, copied JSON configuration,
new socket and data paths, and no public listener. The copied configuration must
resolve its database URL, token-file path, asset roots, world paths, and runtime
namespace to lab-only resources. Never reuse production credentials, namespaces,
volumes, DNS, or player endpoints in the lab.

## Evidence boundary

A lab result proves only the supplied inputs and observed lab behavior. It cannot
prove production cutover readiness, capacity, external routing, backups not used
in the run, or real player recovery. Preserve source commit identity, artifact
checksums, copied-config fingerprint without secrets, commands, final output,
and teardown confirmation.

## Planned automation

No committed clean-room harness provisions this isolation or captures an evidence
bundle. D-OPS owns the follow-up to define reproducible lab provisioning, input
redaction, teardown verification, and an explicit acceptance record. Until then,
perform the procedure manually under the [incident response](incident-response.md)
process and do not describe it as an automated smoke.
