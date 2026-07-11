# Menu data contracts

## Purpose

This file defines the shipped local documentation-menu data contract.

## Status

implemented

## Local route map

| Route family | Source | Enabled effects |
| --- | --- | --- |
| `docs-directory` | bundled docs bundle | open child, prompt search |
| `docs-file` | bundled docs bundle | page turn, open links |
| `docs-links` | bundled docs bundle | open internal link, send external link |
| `docs-search` | bundled docs bundle | open matching file |

Documents are loaded at plugin construction from bundled resources. A local
binding reads no network, database, token file, or daemon data. An enabled row
has local navigation or a documented external-link presentation effect; otherwise
it is inert.

## Withdrawn route families

Server, admin, homes, warps, teleports, shop, adventures, achievements, settings,
claims, profile, and all other daemon-backed route families are withdrawn pending
trusted identity/session attestation. Their bindings, daemon command names,
stale data, grant checks, and mutations must not be packaged in a Java plugin.

## Verification

Tests cover local route reachability, metadata, links, search, pagination, and
malformed local content. Generated route inventory and locale checks cover only
bundled local documentation routes.
