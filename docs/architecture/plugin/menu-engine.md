# Menu engine

## Purpose

This document defines the single source-owned Paper/Folia menu engine.

## Compiled surface

`contracts/menus/README.json` must name exactly `root`, `docs-directory`,
`docs-file`, `docs-links`, and `docs-search`. The compiler and JVM loader both
reject a different set, malformed members, remote dependencies, non-doc dynamic
bindings, authored Back or Close actions, refresh, confirmation, mutation,
unknown navigation targets, missing target parameters, slot collisions, and
missing locale keys.

Paper loads the deterministic bundle and curated docs from its jar. It never
loads a host menu file. Root renders one inert `/lkjmc` guidance item and one
docs navigation item. Dynamic rendering is closed to the four local docs
bindings.

## Session rules

Inventory metadata carries route, session, render revision, slot, and action.
A click applies only when it matches the active per-player adapter and current
frame. Navigation and Back replace the inventory while replacement close events
are suppressed. Explicit Close, ordinary close, reopen, quit, locale change,
and plugin disable retire ownership.

Back uses recorded navigation history when present. A directly opened docs route
projects its compatible parameters to the declared parent. The dynamic action
probe follows directory → file → links → linked file and separately checks Back
from every directly openable docs route.

The action set is `NAVIGATE`, `BACK`, `CLOSE`, and `NONE`; authored slots may use
only navigation or inert None. There is no snapshot view, pending request,
remote response, capability, attestation, or daemon mutation port.

## Verification

Seven deterministic menu probes render every route, compare localized golden
frames, traverse the dynamic docs graph, reject stale metadata, verify explicit
Close, and inspect the shaded jar for removed snapshot and mutation menu classes.
This is candidate adapter evidence, not a live Minecraft client observation.
