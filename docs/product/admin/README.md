# Admin

## Purpose

This area owns operator-facing admin behavior: roles, grants, visibility,
privileged command authorization, and audit trails.

## Table of contents

- [Model](model.md)

## Current status

A role-to-permission catalog, durable grants, revoke/inspect helpers, admin
audit rows, daemon admin commands, and CLI grant/revoke/inspect/audit commands
are implemented. Minecraft visibility from grant snapshots and daemon-wide
privileged mutation enforcement remain target behavior until implemented.

## Contract

Admin behavior must be product-owned rather than scattered through adapters.
Platform permissions and `op` are adapter inputs, but daemon grants are durable
truth for lkjmc admin roles. Privileged daemon mutations must authorize the
end-user or local CLI principal and record safe audit rows.
