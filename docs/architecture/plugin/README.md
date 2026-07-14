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

Java platform modules own lifecycle callbacks and presentation. One shared
Java-common coordinator may fetch revisioned read-only views; it performs all
HTTP work asynchronously on its owned bounded executor. Paper and Velocity must
not create per-player or per-domain pollers. Scheduler callbacks only submit or
read immutable cache views and never wait on database, filesystem, network, or
process work.

Trusted player application, commands, mutations, and transfer adaptation remain
withdrawn. Managed jars come only from verified assets.

## Evidence and degraded behavior

Gradle and real Java 21 HTTP harnesses prove coordinator bounds, repair,
nonblocking submission, and shutdown. Artifact inspection rejects duplicate
pollers, withdrawn commands, registries, mutation/application/transfer bridges,
and credential output. Unsupported behavior remains unavailable rather than a
localized fake mutation.
