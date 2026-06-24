# Paper and Folia plugin

## Purpose

This document defines the target server plugin behavior.

## Responsibilities

- Provide a Folia-safe scheduler bridge.
- Capture and apply player profile snapshots.
- Provide inventory UI and localized player commands.
- Send server heartbeats.
- Run database and daemon operations asynchronously.

## Scheduler rules

Entity mutations run on player or entity schedulers. Region mutations run on
region schedulers. Database, filesystem, network, and process operations never
block scheduler threads.

## Current status

The Paper/Folia plugin is not implemented yet.
