# Documentation browser

## Purpose

This document owns the target in-game documentation browser.

## Current status

No verified `/docs` inventory browser is shipped. The browser described here is
target behavior until a bundled docs index, menu route, command, locales, tests,
and playable smoke are implemented.

## Scope

The browser includes root `README.md`, `AGENTS.md`, and every Markdown file under
`docs/`. It must not expose arbitrary filesystem paths. A development override
may read a configured docs root, but packaged plugin resources must work without
host filesystem access.

## Routes

- `docs-root`
- `docs-dir:<path>`
- `docs-file:<path>:<page>`
- `docs-links:<path>:<page>`
- `docs-search:<query>:<page>`

## Layout

Directory pages list child directories and Markdown files. File pages show ten
wrapped content lines per page and keep Back, Home, Parent, Previous, Next,
Links, Search, and Refresh in stable chrome slots. Wrapping uses conservative
visible width and breaks long code spans or URLs safely.

## Links

Internal Markdown links navigate inside the bundle and anchors jump to the page
containing the heading when possible. Missing internal links render disabled
diagnostics. External URLs send a safe clickable chat component plus copy
fallback only after the player clicks the link item.
