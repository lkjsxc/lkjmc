# Asset architecture

## Purpose

This directory owns contracts for server and plugin artifact storage.


## Status

implemented

## Table of contents

- [Download policy](download-policy.md)
- [Plugin jars](plugin-jars.md)
- [Server jars](server-jars.md)

## Current and target boundary

Assets are immutable, content-addressed files recorded in PostgreSQL. Rust
validation decides eligibility; daemon download and installation adapters do
filesystem and network work. No command may report installation until the file
hash matches trusted metadata.

## Evidence and degraded behavior

Asset commands and registry code are source evidence. Offline or unavailable
upstreams return diagnostics; they never create a verified asset record or
report a completed download.
