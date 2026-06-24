# Daemon

## Purpose

This document defines the target daemon responsibilities.

## API

The daemon exposes Unix socket JSON-RPC for local CLI use and loopback HTTP for
plugins with a local token. Every request has a request ID and actor metadata.

## Responsibilities

- Dispatch command envelopes.
- Reconcile desired and observed instance state.
- Manage local processes.
- Manage jar registry operations.
- Render templates.
- Write audit events for mutating operations.

## Current status

The daemon is not implemented yet.
