# Style tokens

## Purpose

This document records shared item roles and localized labels.

## Status

planned

## Tokens

The compiled route bundle admits only uppercase Bukkit material identifiers and
closed roles: `INFO`, `ACTION`, `NAVIGATION`, `DECORATION`, `DISABLED`,
`SUCCESS`, and `DANGER`. Names and lore are locale keys, not raw transport text.

Generated chrome uses compass or book navigation, arrows for paging, a clock for
refresh, and a barrier for explicit Close. Documentation rows use book and paper
materials through the same renderer.

## Accessibility

Every meaningful item has a nonblank label after MiniMessage color tags are
removed. English and Japanese keys and placeholders match. Color and material
are supplementary cues only.
