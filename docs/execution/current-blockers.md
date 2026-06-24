# Current blockers

## Purpose

This document lists the next executable blockers in priority order.

## Blockers

- [x] Task 00: Repository foundation.
- [x] Task 01: Build foundations.
- [x] Task 02: Core model and config.
- [x] Task 03: PostgreSQL migrations and store.
- [x] Task 04: Daemon API and CLI.
- [ ] Task 05: Local process runtime. Explicit command and verified jar launch,
  periodic reconciliation, process recovery, automatic server-port allocation,
  minimal rendering, stdin stop, and deletion guardrails exist; RCON stop
  remains.
- [ ] Task 06: Jar registry. Local import/list/inspect and launch checksum
  verification exist; PaperMC downloads and pruning remain.
- [ ] Task 07: Installer.
- [ ] Task 08: Java common module.
- [ ] Task 09: Velocity plugin.
- [ ] Task 10: Paper/Folia plugin foundation.
- [ ] Task 11: Inventory UI framework.
- [ ] Task 12: Player profile sync.
- [ ] Task 13: SMP utility imports.
- [ ] Task 14: Proxy utility imports.
- [ ] Task 15: Final hardening.

## Next executable step

Continue Task 05 by adding RCON stop, then continue Task 06 with PaperMC
downloads.
