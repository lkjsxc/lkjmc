# Orchestration architecture

## Purpose

This area owns desired state, presence, autosuspend, temporary instances,
reconciliation, and process runtime behavior.


## Status

implemented

## Table of contents

- [Desired state](desired-state.md)
- [Idle autosuspend](idle-autosuspend.md)
- [Kubernetes runtime](kubernetes-runtime.md)
- [Presence](presence.md)
- [Process runtime](process-runtime.md)
- [Temporary instances](temporary-instances.md)
- [Temporary runtime](temporary-runtime.md)

## Current and target boundary

Instance commands persist durable intent in PostgreSQL. Pure Rust planners
produce effect descriptions; daemon adapters execute process or cluster work
and record observations. Runtime locks must not span PostgreSQL work.

## Evidence and degraded behavior

Planner, reconciler, and runtime-adapter code are source evidence. An adapter
that cannot observe or apply an effect reports its diagnostic or a guarded skip;
it must not invent a running instance or healthy readiness.
