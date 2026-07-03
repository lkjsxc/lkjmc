# Work loop

## Purpose

This document defines the default loop for safe repository changes.

## Steps

1. Read the required files listed in `AGENTS.md`.
2. Pick the named user task or the first incomplete blocker.
3. Update the owner docs for any behavior change.
4. Implement the smallest coherent slice.
5. Run relevant checks.
6. Update `docs/state/README.md`.
7. Commit with truthful `Tested:` and `Not-tested:` trailers when committing.

## Boundaries

Do not create runtime success paths for behavior that has not been implemented.
A failing placeholder is acceptable only when it prevents accidental use.
