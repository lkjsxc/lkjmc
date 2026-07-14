# Menu engine

## Purpose

This document defines the source-owned, document-driven JVM menu architecture.

## Status

implemented

## Selection

Paper has one menu engine. At build time it validates and compiles every route
listed by `contracts/menus/README.json` into a deterministic JVM resource. A
malformed, unindexed, unreachable, cyclic, slot-conflicting, unknown-action, or
locale-incomplete route fails the build. Paper does not load host menu files.

The common pure core owns closed route, dependency, view, action, and failure
types. Paper owns Bukkit inventory rendering and scheduler-safe effects. There
is no generic daemon action, request body, command string, or alternate local
document inventory engine.

## Snapshot boundary

Routes declare typed dependencies on revisioned menu, permission, claim,
settings, profile, routing, or presence snapshots. A dependency is current,
stale, or unavailable. Stale rows remain labelled and inert; unavailable rows
name the failure without pretending data exists. Permission uncertainty denies.

Mutation actions are closed operation identifiers. Dispatch requires a current
permission capability and trusted session attestation. Missing either produces a
localized denial and no request. This task does not add a daemon mutation port.

## Session rules

Inventory metadata carries route, session, request, render revision, slot, and
action. One request may be pending per session. Repeated clicks are denied,
responses for an older request or render are inert, and navigation never closes
the inventory. Only explicit Close closes it.

## Verification

`menuProbes` owns seven deterministic probes and a disposable protocol-like
Paper/Folia inventory harness. It drives production adapter code but is not a
live server or client; external Minecraft remains a guarded lane.
