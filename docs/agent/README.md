# Agent guide

## Purpose

This area defines how coding agents plan, change, verify, resume, and hand off
work in this repository.

## Table of contents

- [Handoff](handoff.md)
- [Work loop](work-loop.md)

## Source map

| Concern | Source |
| --- | --- |
| entry rules and JSON format | `AGENTS.md` |
| task and controller state | `docs/execution/current-blockers.md` |
| shipped behavior | `docs/state/README.md` |
| available checks | `scripts/check-lines.py`, `scripts/check-docs.py`, `scripts/verify-full.sh` |

## Contract

Agents update docs before behavior changes, keep files small, use isolated
worktrees, and report only verification actually run. The controller owns task
transitions; an agent preserves and reports evidence rather than changing
controller state.
