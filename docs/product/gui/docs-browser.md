# Documentation browser

## Purpose

This document owns the in-game documentation browser contract for `/docs` and
Documentation menu actions.


## Status

implemented

## Current status

A generated docs bundle, `/docs` command, main-menu entry, directory/file/search
menus, wrapped file pages, internal/external link actions, deterministic Parent
Directory navigation, and Main Menu return action are implemented. Previous-state
Back history is intentionally not part of the docs browser.

## Scope

The browser includes root `README.md`, `AGENTS.md`, and every Markdown file under
`docs/`. It must not expose arbitrary filesystem paths. A development override
may read a configured docs root, but packaged plugin resources must work without
host filesystem access.

## Routes

- `dir:<path>` lists bundle children below a slash-separated directory path.
- `file:<path>:<page>` renders a Markdown file page.
- `links:<path>:<page>` renders links extracted from a file page.
- `search:<query>` renders bundle search results.

Invalid or stale routes recover to `dir:` and show a safe diagnostic when the
adapter cannot render the requested content.

## Navigation rules

- Main Menu opens the normal lkjmc inventory root.
- Parent Directory derives only from the current route.
- `dir:` has no parent and renders a disabled Parent Directory item.
- `dir:a` resolves to `dir:`.
- `dir:a/b` resolves to `dir:a`.
- `file:a/b.md:0` and `links:a/b.md:0` resolve to `dir:a`.
- `search:<query>` resolves to `dir:` because search has no directory parent.
- Previous and Next are file-page pagination controls only.

## Layout

Directory pages list child directories and Markdown files. File pages show
wrapped content lines with the reading controls next to the content item:

- Slot `21`: Previous page, or an inert disabled Previous item.
- Slot `22`: file content.
- Slot `23`: Next page, or an inert disabled Next item.
- Slot `45`: Main Menu using `NETHER_STAR`.
- Slot `49`: Parent Directory or a disabled parent item at docs root.
- Slot `52`: Links for the current file page.
- Slot `53`: Search instructions.

Previous and Next must not live only in bottom chrome on file pages. Directory,
links, and search pages may keep standard bottom-row pagination chrome.

## Links

Internal Markdown links navigate inside the bundle. Missing internal links render
no action. External URLs send a safe clickable chat component plus copy fallback
only after the player clicks the link item.
