# Admin

## Purpose

This area owns operator-facing admin behavior: roles, grants, visibility,
privileged command authorization, and audit trails.

## Table of contents

- [Model](model.md)

## Current status

A role-to-permission catalog is implemented in Rust core and exposed through
`admin.role.list` plus `lkjmc admin role list`. Durable grants, daemon mutation
enforcement, Minecraft visibility from grants, grant CLI commands, and audit
review remain target behavior until implemented and verified.

## Contract

Admin behavior must be product-owned rather than scattered through adapters.
Platform permissions and `op` are adapter inputs, but daemon grants are durable
truth for lkjmc admin roles. Privileged daemon mutations must authorize the
end-user or local CLI principal and record safe audit rows.
