# Current state

## Purpose

This ledger states what is implemented now. If it conflicts with any target
contract, this file wins for current behavior.

## Implemented

- Repository documentation skeleton is implemented.
- Line-limit checks are implemented in `scripts/check-lines.py`.
- Documentation topology checks are implemented in `scripts/check-docs.py`.
- `scripts/verify.sh` currently runs only the foundation checks.

## Not implemented

- Runtime daemon is not implemented yet.
- CLI is not implemented yet.
- PostgreSQL schema and migrations are not implemented yet.
- Velocity plugin is not implemented yet.
- Paper/Folia plugin is not implemented yet.
- Installer is not implemented yet; `scripts/install.sh` exits with failure.
- Player synchronization is not implemented yet.
- Docker Compose verification is not implemented yet.

## Verification status

The only meaningful acceptance checks are the docs and line checks until build
foundations are added.
