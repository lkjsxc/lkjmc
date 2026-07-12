# Research runs

## Purpose

This index records reproducible observations from one baseline, variant, or
combination execution without converting observations into product claims.

## Table of contents

- [E-CONTRACT bounded run 2026-07-11](e-contract-20260711.md)
- [E-CONTROL re-review Compose run 2026-07-11](e-control-20260711.md)
- [E-DATA correction run 2026-07-11](e-data-20260711.md)
- [E-HC-AUTOMATION run 2026-07-12](e-hc-automation-20260712.md)
- [E-HC-CONTROL Compose run 2026-07-12](e-hc-control-20260712.md)
- [E-HC-PLATFORM high-cost platform run 2026-07-12](e-hc-platform-20260712.md)
- [E-HC-SURFACE local run 2026-07-12](e-hc-surface-20260712.md)
- [E-JVM isolated adapter run 2026-07-11](e-jvm-20260711.md)
- [E-MENU local and candidate evidence run](e-menu-20260712.md)
- [E-NETWORK bounded compiler run](e-network-20260712.md)
- [E-OBS real-boundary run 2026-07-11](e-obs-20260711.md)
- [E-OPS reproducibility run 2026-07-11](e-ops-20260711.md)
- [E-PRODUCT bounded journey run 2026-07-11](e-product-20260711.md)
- [E-QUALITY representative run 2026-07-11](e-quality-20260711.md)
- [E-RUNTIME lifecycle coordinator correction run](e-runtime-20260711.md)
- [E-SECURITY credential experiment run](e-security-20260711.md)
- [F-MEASURE bounded rerun 2026-07-11](f-measure-20260711.md)
- [Run template](run-template.md)

## Evidence rule

Each run records commits, environment, locked toolchain, seed, exact commands,
raw artifact location, result, faults, cleanup, and deviations. Sanitize secrets;
keep raw output in ignored storage. A run harness must derive its root from its
own file, retain its owned bounded raw root until explicit cleanup, and use
portable root-derived rerun commands.

## External boundary

For unavailable systems, record an attempted access, exact missing prerequisite,
a runnable harness, and rerun command. Mark it external proof pending, never
passed or supported.
