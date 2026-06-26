# Plugin architecture

## Purpose

This area owns Java plugin contracts and plugin asset provisioning contracts.

## Table of contents

- [Daemon HTTP](daemon-http.md)
- [Paper and Folia](paper-folia.md)
- [Plugin provisioning](provisioning.md)
- [Third-party plugin policy](third-party-policy.md)
- [Velocity](velocity.md)

## Contract

Plugins must not block scheduler threads on network, database, filesystem, or
process work. Managed plugin jars must be installed from verified assets.
