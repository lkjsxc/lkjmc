# Repository contracts

## Purpose

This area owns repository layout, build ownership, static contract checks, and
functional/effect boundaries. It does not own operational verification outcomes
or controller state.

## Table of contents

- [Build system](build-system.md)
- [Contract checks](contract-checks.md)
- [Functional style](functional-style.md)
- [Layout](layout.md)

## Reading map

| Need | Owner document | Source of truth |
| --- | --- | --- |
| build or package output | [Build system](build-system.md) | `build.gradle.kts`, `Dockerfile`, `docker-compose.yml` |
| deterministic check scope | [Contract checks](contract-checks.md) | `scripts/check-*.py`, `scripts/verify-*.sh` |
| pure planning and effects | [Functional style](functional-style.md) | core plans and adapter traits |
| paths and generated output | [Layout](layout.md) | `.gitignore`, `.dockerignore`, `scripts/check-lines.py` |

## Boundaries

Authored Markdown and source files are limited to 200 lines. Generated outputs
are not committed, but ignore rules do not by themselves exclude a path from
all checks. In particular, `check-lines.py` recursively scans recognized text
files and only skips its listed top-level directories; nested Gradle `build/`
output is a known defect that can fail the check after a build. Treat that as a
verification defect, not evidence that generated output is repository source.
