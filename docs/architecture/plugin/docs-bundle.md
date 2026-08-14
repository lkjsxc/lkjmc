# Docs bundle

## Purpose

This document defines the local resource used by the in-game docs browser.

## Inputs

`contracts/docs-player-corpus.json` is the sole manifest. It currently names six
repository-relative files: root `README.md`, the Minecraft command document, and
four backend-menu documents. `AGENTS.md`, operator runbooks, archive history,
credentials, generated evidence, and every unlisted Markdown file are excluded.

`scripts/build-docs-bundle.py` rejects an unknown manifest member, missing file,
unsafe path, or out-of-corpus internal link target where the parser requires a
bundled target. There is no repository-wide Markdown discovery.

## Output and runtime

The Paper build produces `lkjmc-docs-bundle.json` with normalized paths, titles,
headings, links, source lines, and a directory tree, then packages it inside the
shaded jar. The adapter loads only that resource. It has no host-root override,
filesystem fallback, daemon request, or credential.

Absolute paths, traversal, and unlisted files cannot be opened. An absent or
malformed resource fails plugin construction rather than widening the corpus.
