# Security architecture

## Purpose

This area owns threat boundaries, secret handling, permissions, and safe file
access.


## Status

implemented

## Table of contents

- [Permissions](permissions.md)
- [Secrets](secrets.md)

## Current and target boundary

Minecraft players are untrusted; local shell users in the `lkjmc` group are
privileged. Security policy and authorization decisions are separate from file,
network, and audit effects. Mutating admin actions are audited.

## Evidence and degraded behavior

Authorization, secret, and audit implementations are source evidence. Missing
credentials, denied grants, or unsafe paths fail closed with redacted
 diagnostics; they never expose a secret or report an authorized mutation.
