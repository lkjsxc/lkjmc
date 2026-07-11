# Paper and Folia plugin

## Purpose

This document defines the local-safe Paper and Folia plugin contract.

## Status

implemented

## Shipped responsibilities

- Register only `/menu` and `/docs`.
- Render bundled documentation and local page navigation.
- Maintain the hard-locked local slot-8 documentation token.
- Read only plugin resources and current Bukkit inventory state.

The local UI does not perform database, filesystem, network, download, or
process work. It has no daemon client, token-file reader, product mutation,
claim refresh, profile bridge, or grant cache.

## Withdrawn responsibilities

Daemon-backed commands, profile and claim synchronization, moderation,
heartbeats, dynamic menus, transfer bridges, and Java daemon credentials are
withdrawn pending trusted identity/session attestation. They are absent, not
placeholder features.

## Verification

Source and jar containment inspect registrations, resources, metadata, and all
built jars for withdrawn classes, commands, bridges, and credentials. Local menu
checks prove only the local-safe surface.
