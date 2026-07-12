# E-CONTROL disposable harness

## Purpose

This standalone Cargo package compares disposable execution candidates. It is
research evidence only; it is not a daemon component, command, controller, or
product adoption.

## Table of contents

- [Source layout](src/README.md)

## Safety and health

`LKJMC_LAB_POSTGRES_DISPOSABLE=1` and `LKJMC_LAB_POSTGRES_URL` are mandatory.
The URL must name an `lkjmc_lab_` database on loopback. Compose host `postgres`
also needs `LKJMC_E_CONTROL_COMPOSE=1`. The Rust harness checks `SELECT 1` and
the Python probe retries the real migration command before either workload.
Unsafe or unavailable input prints `BLOCKED` before a child process is started.

## Evidence boundary

The harness creates an owned schema and short-lived local children. The config
probe writes only a temporary config root and uses the same disposable database.
It does not run product migrations against a non-lab database, alter product
source, download assets, print credentials, or change controller state.
