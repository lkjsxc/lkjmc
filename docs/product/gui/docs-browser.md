# Documentation browser

## Purpose

This document owns the planned in-game documentation browser contract for
`/docs` and Documentation menu actions on the menu engine.

## Status

planned

## Scope

The browser includes root `README.md`, `AGENTS.md`, and every Markdown file
under `docs/`. It must not expose arbitrary filesystem paths. A development
override may read a configured docs root, but packaged plugin resources must
work without host filesystem access.

## Routes

`docs-directory` is a `list` route with local binding `docs-directory` and a
required `path` param. It lists child directories and files from the bundled
docs. Directory entries open `docs-directory` with the child path. File entries
open `docs-file` with `path` and `page=0`. `/docs` opens this route with
`path=docs`.

`docs-file` is a `custom` route with local binding `docs-file` and required
`path` and `page` params. Its binding emits positioned custom-view slots for
one Markdown page from `DocPaginator`. Page turns reopen the same route with a
different `page`, so the same-id replace rule prevents reading from inflating
Back history.

`docs-search` is a `list` route with local binding `docs-search` and required
`query` param. It searches the bundle and entries open `docs-file` at the
matching page. Empty results render the true-empty phase with the query echoed
in the info panel.

## Navigation rules

Docs directory routes label slot `49` as Parent Directory, but the action is
still Back. Navigation through the tree pushes directory routes in order. Deep
opens from search reconstruct the needed ancestry before opening the file so
Back walks upward instead of closing.

Main Menu at `45` opens root. Close at `53` closes. Search starts from the
reserved filter row on directory routes and uses the engine text-input prompt.

## File-page slot exception

File pages keep reading controls next to the content item:

- Slot `21`: previous page, or disabled previous.
- Slot `22`: file content item; lore carries wrapped page lines.
- Slot `23`: next page, or disabled next.
- Slot `45`: Main Menu.
- Slot `49`: Back labeled Parent Directory.
- Slot `52`: outbound links for the current page.
- Slot `53`: Close.

This is the only exception to the global pagination row and is encoded as the
`docs-file` custom view.

## Links

Internal Markdown links navigate inside the bundle when the target exists.
Missing internal links render no action. External URLs send a safe clickable
chat component plus copy fallback only after the player clicks the link item.

## Bundle ownership

`DocBundle`, `DocPath`, `DocPaginator`, `DocLineWrapper`, and `DocRoute` remain
the pure docs domain. Bindings consume them directly; the build-time docs bundle
is unchanged.
