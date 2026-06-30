# Web control surface research

## Purpose

This document preserves the research notes that informed the active web control
architecture.

## Promoted contract

The executable contract now lives in [web architecture](../architecture/web/README.md)
and [web operations](../operations/web-control.md). The stable rule from this
research remains that the web UI calls daemon commands instead of inventing a
parallel backend.

## Security notes

The listener binds privately by default, authenticates every request, avoids
printing or exposing daemon tokens, and denies public control unless an operator
explicitly configures a safe front door.

## Verification notes

Default tests should cover daemon command mapping, authentication failures,
private bind configuration, static asset availability, redaction, and audit
coverage for mutating actions.
