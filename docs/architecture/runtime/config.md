# Config

## Purpose

This document defines the implemented JSON configuration model foundation.

## Main config

`lkjmc-core` parses and validates the main `/etc/lkjmc/lkjmc.json` shape with
these sections:

- root paths and socket path
- database connection metadata
- network defaults
- jar registry settings
- local runtime settings

Validation rejects relative product paths, empty names, invalid ports, a fallback
server that is not lowercase kebab-case, a jar User-Agent that does not identify
`lkjmc`, and zero memory or stop timeout values.

## Instance config

`lkjmc-core` parses and validates instance JSON with ID, kind, desired state,
jar reference, ports, memory, template, properties, plugin toggles, and sync
policy. Instance IDs must be lowercase kebab-case.

## Current boundary

The model is pure Rust only. No daemon loader, file watcher, Java schema mirror,
or installer writer exists yet.
