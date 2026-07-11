# Menu framework

## Purpose

This document defines the shipped local documentation inventory implementation.

## Status

partial

Missing: trusted adapter identity and session attestation for daemon menu data
and actions.

## Local document model

The common JVM module loads a bundled documentation JSON file and provides pure
path lookup, search, line wrapping, and pagination helpers. Paper creates
54-slot inventories from that local data. The list shows up to 45 documents;
document pages provide Previous at `46`, Next at `48`, Documentation at `49`,
and Close at `53` when applicable.

## Local effects

The Paper listener recognizes only its own persistent local-document action
metadata. A click opens a bundled path, changes a page, returns to the document
list, or closes the inventory. Unknown metadata is inert. A missing path falls
back to local search; it does not make a network request or fabricate data.

## Withdrawn behavior

The former document/kernel/binding route system, daemon request plans, stale
caches, transfer ports, dynamic rows, confirmations, and mutation actions are
not shipped. The local menu has no daemon client, credential reader, or
scheduler-side I/O path.

## Verification

JVM containment checks production/test source, resources, plugin metadata, and
built jars. Paper Gradle tests load bundled documentation, inspect metadata, and
assert the token slot constant; they do not exercise Bukkit inventory events.
