# Web security

## Purpose

This document defines security requirements for the operator web surface.

## Authentication

The default listener binds to `127.0.0.1`. Except for `/web/login`, every route
requires either a valid session cookie created from the daemon HTTP token source
or an explicit bearer token meant for web use. Static assets served by the
control surface require authentication when they reveal product state.

## Session rules

Session identifiers and CSRF values are generated with strong randomness. Only
non-reversible fingerprints may be stored. Cookies use `HttpOnly`, `SameSite`,
and `Secure` when TLS is enabled by the operator front door.

## Redaction

Rendered pages, JSON responses, logs, audit rows, and errors must not include
daemon bearer tokens, web secrets, database passwords, forwarding secrets,
kubeconfig contents, cookie values, or raw stack traces.

## Authorization

The web adapter submits daemon command envelopes with actor and correlation id.
The daemon remains the final authorization boundary for every mutation.
Unavailable actions render disabled reasons instead of hidden or fake success.
