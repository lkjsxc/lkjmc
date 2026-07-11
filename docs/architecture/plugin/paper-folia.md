# Paper and Folia plugin

## Purpose

This document defines the local-safe Paper and Folia plugin contract.

## Status

implemented

## Shipped responsibilities

- Register only `/menu` and `/docs`.
- Render the bundled documentation browser and its local navigation.
- Maintain the hard-locked local hotbar menu token.
- Use scheduler bridges for Bukkit, Paper, and Folia effects.
- Keep English and Japanese local UI copy in lockstep.

The local UI never blocks a scheduler thread on database, filesystem, network,
or process work. It has no daemon client, token-file reader, product mutation,
claim refresh, profile bridge, or admin grant cache.

## Withdrawn responsibilities

Daemon-backed commands, profile and claim synchronization, moderation,
heartbeats, dynamic menus, transfer bridges, and all Java daemon credentials are
withdrawn pending trusted identity/session attestation. They are not degraded
features and must not be registered as placeholders.

## Verification

Tests cover local docs navigation, hotbar locking, metadata safety, locale
rendering, and scheduler-safe local effects. Source and jar inspection prove
that daemon clients and withdrawn command or bridge classes are absent.

## Proxy target

Paper backends may still use Velocity forwarding configuration supplied by the
runtime. The local-safe plugin does not read forwarding secrets or daemon
credentials.
