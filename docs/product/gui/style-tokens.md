# Style tokens

## Purpose

This document defines platform-neutral visual roles used by Java common.


## Status

implemented

## Roles

- `DECORATION`: blank glass pane, no lore, inert metadata.
- `INFO`: paper, map, book, player head, or clock that summarizes state.
- `NAVIGATION`: compass, arrow, barrier, clock, ender pearl, or map.
- `ACTION`: iconic material that performs a real operation.
- `SUCCESS`: lime dye or lime wool.
- `WARNING`: orange dye or yellow dye.
- `DANGER`: red dye or red wool.
- `DISABLED`: gray dye, barrier, bedrock, or structure void.
- `LOADING`: clock or spyglass with explicit loading copy.
- `SELECTED`: enchanted or named item with selected-state lore.

## Metadata

Visual role never determines behavior by itself. Behavior comes from metadata
and reducer action data. Role only guides rendering, lore order, and tests.

## Accessibility

Names should be concise and localized. Lore should avoid implementation jargon
and should state exact player outcomes, requirements, cooldowns, costs, and
availability.
