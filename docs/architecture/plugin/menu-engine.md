# Menu engine

## Purpose

This document defines the bounded local Paper documentation menu.

## Status

partial

Missing: trusted adapter identity and session attestation for daemon menu data or
mutation effects.

## Active implementation

`com.lkjmc.common.docs` loads the bundled documentation JSON and provides local
path, search, and pagination helpers. `com.lkjmc.paper.LocalDocsMenu` renders
those documents in Paper inventories and owns click handling. Velocity does not
use a menu engine.

The active surface has only a document list, document pages, search, previous,
next, Documentation, and Close. It has no route registry, daemon request plan,
dynamic data binding, profile, admin, shop, exchange, claim, adventure, or
transfer action.

## Effect boundary

The local menu uses only Bukkit inventory effects and bundled resource data. A
click either opens a bundled document, changes a local page, returns to the
document list, or closes the inventory. Malformed metadata is inert; a missing
path returns to local search. No click creates a daemon request or reports a
mutation result.

## Threading and credentials

The shipped menu reads no token file and constructs no daemon client. Minecraft
callbacks perform no database, filesystem, network, download, or process work.
No Java credential, daemon bridge, or scheduler hop is available to restore a
withdrawn action.

## Verification

Local documentation and hotbar tests, menu checks, and JVM containment inspect
the source and built jars for only this local surface. They do not prove a
daemon-backed menu row.
