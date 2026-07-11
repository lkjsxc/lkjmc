# Navigation

## Purpose

This document defines local documentation navigation.

## Status

implemented

## Behavior

`/menu` and the hotbar token open the local document list. `/docs [path]` opens
a normalized bundled path; `/docs search <query>` lists local search matches.
Selecting a document opens paged content. Previous and Next page controls remain
within that file, Documentation returns to the list, and Close closes the
inventory.

## Safety

Only local bundled paths are opened. Unknown paths become a local search; they
do not read the filesystem, call the daemon, or run a player command. Clicks in
the docs inventory are cancelled before navigation is applied.

## Verification

Local docs and containment checks cover path normalization, paging, local search,
and absence of daemon actions.
