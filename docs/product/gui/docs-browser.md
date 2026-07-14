# Documentation browser

## Purpose

This document owns the `/docs` routes inside the selected menu engine.

## Status

implemented

## Corpus

The build bundles only `contracts/docs-player-corpus.json`. Paths are normalized
repository-relative identifiers. There is no development override, arbitrary
Markdown discovery, host read, daemon fallback, credential, generated evidence,
or operator-secret document.

## Routes

`docs-directory` lists curated files, `docs-file` pages wrapped lines,
`docs-links` lists safe in-corpus links, and `docs-search` searches the curated
bundle. Missing paths and empty results produce localized rows. Previous, Next,
Back, parent, and Close are normal typed actions in the same runtime.

## Safety

A documentation link never authorizes a product action. Unknown metadata and
out-of-corpus paths are inert with localized chat fallback. Only explicit Close
closes the inventory.

## Verification

Route coverage, locale goldens, no-unintended-close, and protocol menu sequences
include documentation navigation. This deterministic harness is not a live
Minecraft client.
