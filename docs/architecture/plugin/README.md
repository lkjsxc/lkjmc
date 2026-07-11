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

Java plugins own platform callbacks, menus, and daemon-client adaptation. They
must not block scheduler threads on network, database, filesystem, or process
work; daemon and asset adapters own those effects. Managed jars come only from
verified assets.

## Evidence and degraded behavior

JVM plugin modules and their tests are source evidence. A failed daemon call,
missing asset, or unsupported platform path gives localized failure feedback and
must not claim a completed mutation.
