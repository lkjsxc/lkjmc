# Docs bundle

## Purpose

This document defines the local plugin architecture for the in-game docs browser.


## Status

implemented

## Current status

The Paper build generates `lkjmc-docs-bundle.json` from repository Markdown and
packages it as a plugin resource. The Paper docs menu loads this bundled local
resource at construction and renders local paths or search results; it has no
daemon fallback or credential.

## Inputs

- Root `README.md`.
- Root `AGENTS.md`.
- Every nonarchive Markdown file under `docs/`.

Archive history remains in the repository but is excluded from the shipped
resource so withdrawn class and credential names cannot enter a plugin jar.

## Output

The build produces a JSON resource containing normalized paths, titles, headings,
links, source lines, and a directory tree. The resource is packaged into the
Paper plugin jar. Generated files stay out of commits unless the repository adds
a documented generated-source policy.

## Runtime

The Paper adapter loads the packaged resource during plugin construction and
opens only normalized bundled paths. It rejects an absent or invalid resource
without a fallback daemon call. No `LKJMC_DOCS_ROOT` override is shipped; path
normalization rejects absolute paths, traversal, and external file paths.
