# Security architecture

## Purpose

This area owns threat boundaries, secret handling, permissions, and safe file
access.


## Status

implemented

## Table of contents

- [Permissions](permissions.md)
- [Secrets](secrets.md)

## Contract

Minecraft players are untrusted. Local shell users in the `lkjmc` group are
privileged. All mutating admin actions are audited.
