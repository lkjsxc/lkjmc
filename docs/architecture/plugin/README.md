# Plugin architecture

## Purpose

This area owns Java plugin contracts and plugin asset provisioning contracts.

## Status

implemented

## Table of contents

- [Daemon HTTP](daemon-http.md)
- [Docs bundle](docs-bundle.md)
- [Menu engine](menu-engine.md)
- [Paper and Folia](paper-folia.md)
- [Plugin provisioning](provisioning.md)
- [Third-party plugin policy](third-party-policy.md)
- [Velocity](velocity.md)

## Current and target boundary

Java plugins own only local platform callbacks and presentation: Paper `/menu`,
`/docs`, hotbar/docs UI, and Velocity MOTD/tab-list. Daemon-client adaptation is
withdrawn pending trusted identity/session attestation. Local callbacks must not
block scheduler threads on network, database, filesystem, or process work;
managed jars come only from verified assets.

## Evidence and degraded behavior

JVM plugin modules and their tests are source evidence. Artifact inspection must
prove no daemon client, credential reader, withdrawn command, registry, or bridge
is packaged. Unsupported daemon behavior remains unavailable rather than a
localized fake mutation.
