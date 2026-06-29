# Admin

## Purpose

This area owns operator-facing admin behavior: roles, grants, visibility,
privileged command authorization, and audit trails.

## Table of contents

- [Model](model.md)

## Current status

A coherent product admin model is not shipped yet. Existing runtime checks use
platform permissions and operator defaults. Treat daemon-enforced admin grants,
Minecraft visibility from grants, CLI grant management, and audit review as
target behavior until implemented and verified.

## Contract

Admin behavior must be product-owned rather than scattered through adapters.
Platform permissions and `op` are adapter inputs, but daemon grants are durable
truth for lkjmc admin roles. Privileged daemon mutations must authorize the
end-user or local CLI principal and record safe audit rows.
