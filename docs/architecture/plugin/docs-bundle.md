# Docs bundle

## Purpose

This document defines target plugin architecture for the in-game docs browser.


## Status

implemented

## Current status

The Paper build generates `lkjmc-docs-bundle.json` from repository Markdown and
packages it as a plugin resource. Fallback diagnostics and the Paper docs menu
adapter are implemented in source; playable smoke proof remains outstanding.

## Inputs

- Root `README.md`.
- Root `AGENTS.md`.
- Every Markdown file under `docs/`.

## Output

The build produces a JSON resource containing normalized paths, titles, headings,
links, source lines, and a directory tree. The resource is packaged into the
Paper plugin jar. Generated files stay out of commits unless the repository adds
a documented generated-source policy.

## Runtime

The Paper adapter loads the packaged resource asynchronously during startup or
first use and exposes typed diagnostics if it is missing or invalid. A developer
may set `LKJMC_DOCS_ROOT` for local override, but path normalization must reject
absolute paths, traversal, and links outside the allowed tree.
