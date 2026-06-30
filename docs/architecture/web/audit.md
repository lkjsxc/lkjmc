# Web audit

## Purpose

This document defines audit behavior for web-initiated control actions.

## Event shape

Every mutating web route writes or delegates a durable audit event with actor,
command, target id when present, correlation id, result class, and safe reason.
Audit payloads must never contain submitted secrets or credential headers.

## Command delegation

When a daemon command already writes the product audit event, the web layer adds
only a safe web access event if needed. The product mutation is not duplicated in
PostgreSQL outside the daemon handler.

## Failure handling

Authentication failures are counted without echoing submitted values.
Authorization denials name the command family and target class. Dependency
failures name the dependency class and next operator action.
