# Menu framework

## Purpose

This document defines the planned platform-neutral menu engine.

## Status

planned

## Engine overview

Menu structure is data. One JSON document per route defines title, theme, size,
params, parent hint, chrome, static slots, list grammar, data binding, and
confirmation reason. Documents are loaded from bundled resources at plugin
enable; any invalid document fails enable rather than pretending menus work.

The common JVM engine has four parts:

- Documents: immutable route descriptions and structural validation.
- Kernel: immutable model, sealed messages, total `update`, and total `frame`.
- Bindings: pure decoders from daemon or local data into route views.
- Runtime: the Paper adapter that renders frames and executes effects.

## Pure kernel

`update` receives the document set, current model, and one message, then returns
a new model plus effects. It never performs I/O, never constructs platform
types, and never reads time or randomness directly. Session ids and any clocks
come from runtime-supplied inputs so tests stay deterministic.

`frame` renders every document, phase, and page into a title, size, and slots.
Every non-inert slot carries complete metadata: route, params, slot, action key,
payload, session, and epoch. Text is `TextRef`, not resolved strings.

## Binding contract

Bindings are pure. A daemon binding returns a request plan and decodes the JSON
response into a list, detail, or custom view. A local binding reads an in-memory
source such as the docs bundle. Asynchrony, stale data, permissions snapshots,
and player scheduler hops belong to the runtime.

## Action and effect vocabulary

Document actions are open, Back, Close, Refresh, player command, daemon command,
text input, and inert. Binding entries may also request transfer or message
effects. A decision that does no work returns an empty effect list; there is no
separate effect for doing nothing.

Specialized hard-coded aliases for selection, buying, and setting flips are not
part of the engine. Their real behavior is expressed as daemon actions with
route params, payload metadata, and declarative success or failure copy.

## Runtime boundary

The Paper runtime owns inventories, metadata codecs, event cancellation,
schedulers, daemon HTTP, player messages, transfers, text prompts, stale cache,
and hotbar entrypoints. It may not decide navigation or frame content outside
the kernel.

Scheduler threads never block on database, filesystem, network, downloads, or
process work. Daemon completions re-enter the correct player scheduler before
dispatching a data message.

## Verification

Unit tests cover update rows, frame totality, document validation, binding
registry parity, and metadata failure classes. Contract checks keep documents,
locale keys, daemon commands, bindings, and generated route docs in lockstep.
