# Documentation browser

## Purpose

This document owns the four local `/docs` routes.

## Corpus

The build bundles only `contracts/docs-player-corpus.json`. Paths are normalized
repository-relative identifiers. There is no development override, arbitrary
Markdown discovery, host read, daemon fallback, credential, generated evidence,
or operator-secret document.

## Routes

`docs-directory` lists curated files, `docs-file` pages wrapped lines,
`docs-links` lists safe in-corpus links, and `docs-search` searches the curated
bundle. Missing paths and empty results produce inert localized rows. Previous,
Next, Back, Main Menu, and Close remain local typed actions.

A docs link never authorizes a product effect. Out-of-corpus paths cannot read
the host filesystem or call the daemon.

## Verification

Route coverage, locale goldens, no-unintended-close, and stale-render probes
include docs navigation. This deterministic evidence is not a live player click.
