# Architecture views

## Purpose

This directory provides traceable cross-cutting views of the implemented
control plane. They complement owner contracts; they do not redefine them.

## Status

implemented

## Table of contents

- [Deployment](deployment.md)
- [Domain to effect](domain-to-effect.md)
- [Execution](execution.md)
- [Identity](identity.md)
- [Observability](observability.md)
- [Workflow and change propagation](workflow-change-propagation.md)

## Reading rule

Each view names source files that establish its claims and its non-atomic
boundaries. PostgreSQL transactions cover only database work; no view implies
that a process, filesystem, network, JVM, or cluster effect joined a database
transaction.

## Source-path corrections

The prior review named six stale files. Their current paths are:

| Stale path | Correct path |
| --- | --- |
| `crates/lkjmc-daemon/src/plugin_downloads.rs` | `crates/lkjmc-daemon/src/assets/plugin_downloads.rs` |
| `crates/lkjmc-daemon/src/plugin_install.rs` | `crates/lkjmc-daemon/src/assets/plugin_install.rs` |
| `crates/lkjmc-daemon/src/api.rs` | `crates/lkjmc-daemon/src/dispatch.rs` |
| `crates/lkjmc-daemon/src/status_api.rs` | `crates/lkjmc-daemon/src/commands/status_api.rs` |
| `crates/lkjmc-daemon/src/doctor_api.rs` | `crates/lkjmc-daemon/src/commands/doctor_api.rs` |
| `crates/lkjmc-daemon/src/web_routes.rs` | `crates/lkjmc-daemon/src/web/routes.rs` |
