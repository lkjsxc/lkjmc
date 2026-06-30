# Full implementation pass

## Purpose

This task plan records the user-requested completion pass over promoted product
surfaces.

## Order

1. Reconcile stale documentation and promote web and Kubernetes scope.
2. Add deterministic drift checks and guarded smoke scripts for promoted areas.
3. Complete daemon HTTP token rotation in daemon, CLI, audit, and tests.
4. Enable public wake-and-join controls with expiry, cancellation, cleanup, and
   transfer safety.
5. Productize End Expedition as a shop/menu executor that charges exactly once.
6. Add JVM runtime config validation and Rust-to-Java schema drift coverage.
7. Add the authenticated web control surface backed by daemon commands.
8. Add the Kubernetes runtime adapter with manifest planning and real effects.
9. Refresh the current-state ledger only after implementation and verification.

## Commit rule

Documentation contract changes land before dependent source changes. Each source
slice lands with the tests or guarded smoke that prove it.

## Verification

Run the narrowest relevant checks after each slice. Final handoff reports
`./scripts/check-lines.py`, `./scripts/check-docs.py`, drift checks, Rust checks,
Gradle checks, Compose verification when available, and skipped live smokes.
