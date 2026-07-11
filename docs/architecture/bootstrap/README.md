# Bootstrap architecture

## Purpose

This directory owns the target architecture for turning a clean installation
into a playable managed network.


## Status

implemented

## Table of contents

- [Desired network](desired-network.md)
- [Effects](effects.md)
- [Planner](planner.md)
- [Rollback](rollback.md)

## Current and target boundary

Bootstrap decisions belong in pure Rust core. Daemon adapters gather facts,
apply effects, record steps, and report diagnostics; they do not make planning
a filesystem, database, process, or network side effect.

## Evidence and degraded behavior

Bootstrap planners, effect variants, and daemon handlers are source evidence.
A failed or unavailable prerequisite remains a failed or skipped step with its
real diagnostic; it is never a completed bootstrap effect.
