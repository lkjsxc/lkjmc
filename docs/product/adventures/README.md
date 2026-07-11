# Adventures

## Purpose

This area owns short-lived generated-server adventure contracts.


## Status

implemented

## Table of contents

- [Catalog](catalog.md)
- [Lifecycle](lifecycle.md)
- [End expedition](end-expedition.md)

## Contract

Adventure purchases are daemon-side, atomic, catalog-driven, and backed by real
temporary instances before any live menu, command, or shop action reports
success. End Expedition is one catalog entry, not a special product layer.

## Outcome, journey, and evidence boundary

A player selects an enabled catalog entry, confirms its cost and party scope,
and transfers only after the temporary backend is ready. Validation failure
spends nothing; later startup, readiness, registration, or first-transfer
failure records failure and performs the idempotent refund path. Store and
adapter tests support these paths; they do not prove a live temporary instance
or transfer without an opt-in playable run.
