# Docs bundle

## Purpose

This document defines target plugin architecture for the in-game docs browser.

## Current status

No generated docs bundle is shipped yet. The bundle and browser route are target
behavior until the generator, packaged resource, fallback diagnostics, and Paper
menu adapter are implemented.

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
