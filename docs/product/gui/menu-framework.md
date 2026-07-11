# Menu framework

## Purpose

This document defines the implemented platform-neutral menu engine.

## Status

implemented

## Engine overview

Menu structure is data. One JSON document per route defines title, theme, size,
params, parent hint, chrome, static slots, list grammar, data binding, and
confirmation reason. `contracts/menus.schema.json` records the structural JSON
shape, while `check-menus.py` enforces semantic rules for actions, bindings,
permissions, locale keys, reachability, and generated route docs. Documents are
loaded from bundled resources at plugin enable; any invalid document fails enable
rather than pretending menus work.

The common JVM engine has four parts:

- Documents: immutable route descriptions and structural validation.
- Kernel: immutable model, sealed messages, total `update`, and total `frame`.
- Bindings: pure decoders from bundled local documentation into route views.
- Runtime: the Paper adapter that renders local frames and executes local effects.

## Pure kernel

`update` receives the document set, current model, and one message, then returns
a new model plus effects. It never performs I/O, never constructs platform
types, and never reads time or randomness directly. Session ids and any clocks
come from runtime-supplied inputs so tests stay deterministic.

`frame` renders every document, phase, and page into a title, size, and slots.
Every non-inert slot carries complete metadata: route, params, slot, action key,
payload, session, and epoch. Text is `TextRef`, not resolved strings.

## Binding contract

Bindings are pure. The shipped local binding reads the in-memory docs bundle and
decodes it into a list, detail, or custom view. Custom views carry positioned
frame slots, not platform items. Daemon request plans, stale data, permission
snapshots, and daemon scheduler hops are withdrawn.

## Action and effect vocabulary

Shipped document actions are open, Back, Close, local text input, external-link
presentation, and inert. A decision that does no work returns an empty effect
list; there is no separate effect for doing nothing. Player commands, daemon
commands, transfer effects, buying, selection, and settings mutations are
withdrawn.

## Runtime boundary

The Paper runtime owns inventories, metadata codecs, event cancellation,
schedulers, local messages, text prompts, and hotbar entrypoints. It may not
decide navigation or frame content outside the kernel. Scheduler threads never
block on database, filesystem, network, downloads, or process work. The shipped
runtime has no daemon completion path.

## Verification

Unit tests cover local update rows, frame totality, document validation, docs
binding parity, and metadata failure classes. Contract checks keep bundled local
documents, locale keys, bindings, and generated route docs in lockstep.
