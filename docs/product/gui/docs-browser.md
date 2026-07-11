# Documentation browser

## Purpose

This document owns the shipped local `/docs` and documentation-menu contract.

## Status

implemented

## Scope

The Paper plugin loads `lkjmc-docs-bundle.json` packaged with the plugin. It
opens only normalized bundled paths and searches only that bundled content. It
has no development filesystem override, daemon fallback, credential, or player
profile lookup.

## Local navigation

`/docs` opens a local path when present and otherwise opens local search.
`/menu` and the slot-8 token open the local document list. A document page shows
wrapped bundled lines, Previous, Next, Documentation, and Close. The list and
pages use only plugin-local persistent metadata; unknown metadata is inert.

## Failure behavior

An absent bundled path falls back to local search. An invalid bundle prevents the
local surface from loading rather than exposing host files or inventing a daemon
result. Links and search results never authorize a product action.

## Verification

Bundle generation and the Paper local-surface test cover the bundled resource,
plugin metadata, and token constant. `check-menus.py` validates only repository
catalog metadata, while JVM containment excludes withdrawn surfaces. These
checks do not prove rendering, a dynamic route, or daemon-backed documentation
data.
