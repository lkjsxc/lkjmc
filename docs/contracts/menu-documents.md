# Menu documents

## Purpose

This document defines the authored JSON route contract and compiled JVM bundle.

## Current catalog

`contracts/menus/README.json` indexes exactly 62 route documents. Together the
index and routes are the 63 menu JSON documents. The catalog includes the root,
network, travel, claims, economy, social, profile, settings, staff, adventure,
and documentation families.

## Closed shape

Every route has exact members for identity, kind, locale title, theme, inventory
size, parameters, parent, typed dependencies, chrome, source slots, dynamic
binding, and confirmation reason. Actions are only `NAVIGATE`, `BACK`, `CLOSE`,
`REFRESH`, `NONE`, or `MUTATION`. A mutation names one closed operation and one
capability; generic daemon actions, command strings, and request bodies are
forbidden.

Dependencies name a generated typed sync domain and key scope. Slot numbers,
parent edges, route targets, required parameters, action members, binding names,
and locale keys are exact. All routes must be reachable from `root` and parent
chains must terminate there.

## Compilation

`scripts/compile-menu-bundle.py` validates JSON, both locale catalogs, graph and
slot invariants, and writes stable compact JSON ordered by route id. Gradle
compiles a candidate and compares it with the source-owned
`platforms/jvm/common/src/generated/resources/lkjmc-menu-bundle.json`.

## Documentation corpus

The docs bindings may expose only paths in
`contracts/docs-player-corpus.json`. Repository internals, secrets, generated
evidence, and arbitrary host paths are not player content.

## Change procedure

Edit the route and both locales, regenerate route Markdown and the JVM bundle,
then run `check-menus.py`, Gradle tests, and `menuProbes`.
