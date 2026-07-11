# Work loop

## Purpose

This document defines the default loop for safe, resumable repository changes.

## Steps

1. Create or enter the isolated worktree named by the controller and inspect its
   branch, commit, and status.
2. Read the required files in `AGENTS.md`, then the named task and prior
   evidence. Do not infer a task transition from an incomplete worktree.
3. Update owner docs before any behavior change. For JSON, use the formatting
   and validation contract in `AGENTS.md`.
4. Implement the smallest coherent slice; keep pure planning separate from
   effect adapters.
5. Run the narrowest relevant checks and record exact outcomes, including skips.
6. Update shipped state only when implementation and evidence support it; do not
   alter controller claims, completion, or routing state.
7. Commit the coherent slice with truthful `Tested:` and `Not-tested:` trailers.
8. Hand off the commit, evidence, risks, and one next executable step.

## Resumption boundary

A resumed agent rereads the task, owner docs, worktree status, and previous
handoff before editing. It continues from recorded evidence or reports the
blocker; only the controller changes task-ledger state.

## Effect boundary

Do not create runtime success paths for behavior that has not been implemented.
A failing placeholder is acceptable only when it prevents accidental use.
