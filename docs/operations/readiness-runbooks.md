# Readiness runbooks

## Purpose

This planned document records ownership and boundaries for capacity, support
bundle, and destructive cutover operations that lack committed automation.

## Status

planned

## Capacity

No product capacity model, load generator, saturation threshold, or admission
control proof is committed. D-OPS must define a workload, resource metrics,
pass/fail thresholds, and evidence retention before a capacity result is claimed.
A successful smoke proves neither player capacity nor sustained availability.

## Support bundle

No support-bundle command exists. Do not assemble token files, raw configs,
databases, worlds, or unbounded logs into an ad-hoc archive. D-OPS must define a
redacted bounded bundle, access policy, manifest, and secret scan before a
support-bundle runbook becomes implemented. Until then use the redacted evidence
rules in [incident response](incident-response.md).

## Destructive cutover

No transactional cutover or automatic rollback workflow is committed. A future
runbook must require a clean-room restore, named rollback decision point,
maintenance ownership, backups, config and artifact identities, route validation,
and teardown or rollback evidence. Do not call a manual stop/start sequence a
safe cutover.

## Follow-up

The coverage entries for this document and [clean-room lab](clean-room-lab.md)
retain `D-OPS` follow-up ownership. They are planned documentation boundaries,
not controller state changes or implementation claims.
