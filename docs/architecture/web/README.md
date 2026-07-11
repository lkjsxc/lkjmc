# Web architecture

## Purpose

This area owns the private authenticated operator web surface.


## Status

implemented

## Table of contents

- [Routes](routes.md)
- [Security](security.md)
- [Audit](audit.md)

## Current and target boundary

The web listener is a private presentation adapter over daemon commands. It
does not write product state directly or expose secrets; daemon authorization,
command planning, store, and runtime adapters retain their ownership.

## Evidence and degraded behavior

Daemon web routes and guarded web smoke are source evidence. Missing login,
CSRF, daemon, or browser prerequisites deny or skip the operation with a real
diagnostic; they never render a successful mutation.
