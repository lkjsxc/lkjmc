# Menu documents

## Purpose

This document defines the source-owned contract for the supported backend menu.

## Catalog

`contracts/menus/README.json` indexes exactly five route documents:

- `root`;
- `docs-directory`;
- `docs-file`;
- `docs-links`;
- `docs-search`.

The compiler rejects an incomplete index, unknown parent or navigation target,
missing required navigation parameter, duplicate slot, chrome collision,
missing locale key, remote dependency, refresh control, confirmation, mutation,
unsupported dynamic binding, or any route count other than five.

## Actions and data

Authored slots may use only `NAVIGATE` or `NONE`. The renderer adds only Back
and Close where the route chrome requests them. Dynamic regions may bind only
to the four local docs renderers. Docs content comes from the curated files in
`contracts/docs-player-corpus.json` and is bundled into the Paper jar.

No route accepts an operation, capability, generic body, daemon command,
snapshot view, remote refresh, or confirmation action.

## Generated consumers

`scripts/compile-menu-bundle.py` writes the deterministic
`lkjmc-menu-bundle.json` consumed by Java common. `scripts/generate-menu-docs.py`
writes the route catalog under `docs/product/gui/routes/`.
`platforms/jvm/common/src/main/java/com/lkjmc/common/menu/MenuBundle.java` repeats
the five-route and closed-action checks at the JVM boundary.

The deterministic menu probe renders every route and inspects the shaded Paper
jar. Live player acceptance is separate and remains required.
