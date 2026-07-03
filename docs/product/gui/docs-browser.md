# Documentation browser

## Purpose

This document owns the in-game documentation browser contract for `/docs` and
Documentation menu actions.

## Status

partial

Missing: full reuse of the shared menu chrome helper in the raw Bukkit docs
browser renderer.

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

Docs browser uses route-derived Parent Directory, not route-stack Back. Main
Menu opens the normal lkjmc inventory root with `NETHER_STAR`. `dir:` has no
parent and renders a disabled Parent Directory item. Files and links return to
their containing directory. Search returns to docs root.

## Layout

Docs surfaces use the shared border and stable controls. Slot `49` is Parent
Directory. File pages keep reading controls next to the content item:

- Slot `21`: Previous page, or disabled Previous.
- Slot `22`: file content.
- Slot `23`: Next page, or disabled Next.
- Slot `45`: Main Menu.
- Slot `49`: Parent Directory.
- Slot `52`: Links for the current file page.
- Slot `53`: Search instructions.

Directory, links, and search pages may also use bottom-row pagination chrome.

## Links

Internal Markdown links navigate inside the bundle. Missing internal links render
no action. External URLs send a safe clickable chat component plus copy fallback
only after the player clicks the link item.

## Verification

Render tests assert docs root and file pages have the shared border, Main Menu,
Parent Directory, and inert decoration metadata.
