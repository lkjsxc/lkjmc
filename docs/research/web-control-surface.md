# Web control surface research

## Purpose

This concise history records research that informed the shipped web control
surface; it is not the current behavior contract.

## Current owner evidence

The executable contract and current limits live in [web architecture](../architecture/web/README.md),
[web operations](../operations/web-control.md), and
[surfaces state](../state/surfaces.md). The web UI calls daemon commands rather
than inventing a parallel backend.

## Security notes

The listener binds privately by default, authenticates every request, avoids
printing or exposing daemon tokens, and denies public control unless an operator
explicitly configures a safe front door.

## Research boundary

The owner docs define shipped checks. Any browser or public-front-door candidate
needs a real authenticated harness, sanitized artifacts, and an external-access
prerequisite record when it cannot run. A planned probe is not web support.
