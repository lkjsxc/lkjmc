# Menu engine

## Purpose

This document defines the planned JVM menu engine architecture, package layout,
startup loading, threading contract, and Paper renderer rules.

## Status

planned

## Layers

`contracts/menus/*.json` defines menu documents. JVM common loads and validates
those documents, runs the pure kernel, and decodes route data through pure
bindings. Paper owns the runtime adapter and all Bukkit or Folia effects.
Velocity does not depend on the engine.

Dependency direction is fixed:

1. `com.lkjmc.common.ui.document`
2. `com.lkjmc.common.ui.kernel`
3. `com.lkjmc.common.ui.binding`
4. `com.lkjmc.paper.ui`

`document`, `kernel`, and `binding` import no Bukkit, Paper, Folia, Velocity,
network, filesystem, database, scheduler, download, or process APIs. Gson and
JDK collection types are allowed. `binding` may depend on document and kernel
types; document and kernel never depend on bindings.

## Document package

`com.lkjmc.common.ui.document` owns immutable document types and validation:
`MenuDocument`, `MenuDocumentSet`, `MenuDocumentLoader`,
`MenuDocumentValidator`, `DocumentAction`, `StaticSlot`, `ListGrammar`,
`ChromeSpec`, and `RegionCatalog`.

`MenuDocumentLoader.fromResources()` reads `/menus/*.json` from the bundled JVM
resource set. The document set is constructed once at Paper plugin enable and
contains derived indices such as children by parent and entrypoints.

Any parse or validation error is a plugin-enable failure. A network with a
broken menu contract must fail loudly instead of silently skipping routes.

## Kernel package

`com.lkjmc.common.ui.kernel` owns `UiModel`, `UiMsg`, `UiStep`, `UiUpdate`,
`UiFrame`, `FrameSlot`, `UiEffect`, `RoutePhase`, `RouteView`, `EntryView`,
`TextRef`, `MenuMetadata`, `MenuFailureCode`, and `Pagination`.

The kernel exposes one total `update` function and one total `frame` function.
It performs no I/O, reads no clock, creates no random values, and constructs no
platform objects. Exhaustive switches over sealed engine types use no default
branch so adding a variant breaks compilation.

## Binding package

`com.lkjmc.common.ui.binding` owns `MenuBinding`, `BindingContext`,
`BindingRegistry`, `DaemonRequestPlan`, and one or more small files per domain.
A binding plans daemon reads or local reads and decodes responses into
`RouteView`. Asynchrony and scheduler hops are runtime responsibilities.

## Paper runtime package

`com.lkjmc.paper.ui` owns `UiSessionService`, `UiInventoryListener`,
`UiRenderer`, `UiEffectRunner`, `UiMetadataCodec`, `UiTextInput`,
`UiStaleCache`, and `UiEntrypoints`.

`UiSessionService` is the only dispatch pipeline: load the player's model, call
`UiUpdate.update`, store the new model, run effects, then render. It always runs
on the owning player's scheduler thread. Sessions clear on matching inventory
close and on quit.

`UiInventoryListener` cancels top-inventory clicks and drags, decodes metadata,
and dispatches messages. Bottom-inventory clicks pass through except for the
hotbar token rules.

`UiEffectRunner` performs daemon HTTP, commands, transfers, messages, text
prompts, close requests, and stale-cache lookup. Daemon completions re-enter the
player scheduler before decoding or dispatching.

## Renderer rules

`UiRenderer` resolves `TextRef` through `MessageCatalog` and MiniMessage, builds
Adventure Components, writes PDC metadata through `UiMetadataCodec`, and applies
frames to inventories.

If the player already has an engine inventory for the same session and the
frame size is unchanged, the renderer mutates item stacks in place and does not
call `openInventory`. A full open happens only on route change, size change, or
when no engine inventory is open. Titles are route-constant by design.

The inventory holder stores only the session id. Documents hold structure and
the session service holds state.

## Threading contract

Scheduler threads never perform database, filesystem, network, download, or
process work. Daemon HTTP is async with a bounded timeout. `update` and `frame`
run only on the owning player's scheduler thread, making each player model
single-writer without locks.

## Entry points

`/menu`, the hotbar token, and `/docs` all dispatch `Open` messages through
`UiEntrypoints`. Existing hotbar token protection and inventory repair stay in
place and retarget this entrypoint service.

## Verification

Architecture tests reject forbidden imports in pure packages. Document tests
load every bundled menu document. Runtime tests cover in-place refresh,
metadata validation, scheduler re-entry, and close-only-via-close-slot behavior.
