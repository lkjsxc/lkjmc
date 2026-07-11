# Permission coverage

## Purpose

This document maps local-safe Paper permission metadata to checked
documentation.

## Source owner

- Paper metadata: `platforms/jvm/paper/src/main/resources/plugin.yml`.

## Checked docs

- Permission contract: [../architecture/security/permissions.md](../architecture/security/permissions.md).
- Minecraft command mapping: [../product/commands/minecraft.md](../product/commands/minecraft.md).

## Identity and proof boundary

The two local UI permission names are capability labels, not authenticated
daemon identity. The command-envelope actor and caller-provided
`platformPermission` value remain untrusted. Paper/Folia and Velocity daemon
adapters are withdrawn pending trusted identity/session attestation.

`check-permissions.py` proves only name parity between Paper metadata and the
permission owner document. `check-jvm-containment.py` proves no daemon permission
resolver or admin registration is packaged.
