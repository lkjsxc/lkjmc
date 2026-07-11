# Admin

## Purpose

This area owns operator-facing roles, grants, privileged daemon authorization,
and audit trails.

## Status

implemented

## Table of contents

- [Model](model.md)
- [Menus](menus.md)
- [Commands](commands.md)

## Current status

The role-to-permission catalog, durable grants, revoke/inspect helpers, audit
rows, daemon admin commands, and CLI/web grant, revoke, inspect, and audit
controls are implemented. Minecraft `/lkjmc`, Java grant snapshots, and Admin
inventory menus are withdrawn pending trusted identity/session attestation.

## Contract

Daemon grants are durable truth for lkjmc admin roles. Web and CLI use their
attested identities; Paper and Velocity do not request grants or expose admin
controls. Privileged daemon mutations authorize an end-user or local operator
principal and record safe audit rows.

## Outcome, journey, and evidence boundary

An operator uses an attested CLI or web control, supplies required context and
confirmation, then receives a durable result or a redacted diagnostic. Cached
grants, platform permissions, and `op` never provide Java daemon authority.
Command and audit tests support this repository claim; they do not prove a live
external identity-provider session.
