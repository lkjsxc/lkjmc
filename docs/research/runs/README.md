# Research runs

## Purpose

This index records reproducible observations from one baseline, variant, or
combination execution without converting observations into product claims.

## Table of contents

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
