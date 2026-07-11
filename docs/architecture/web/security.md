# Web security

## Purpose

This document defines security requirements for the operator web surface.


## Status

implemented

## Authentication

The default listener binds to `127.0.0.1`. Except for `/web/login`, every route
requires either a valid session cookie created from the daemon HTTP token source
or an explicit bearer token meant for web use. Static assets served by the
control surface require authentication when they reveal product state.

## Session rules

Session identifiers and CSRF values are generated with strong randomness. Only
non-reversible session, token, and CSRF fingerprints are stored; the CSRF value
is derived from a private in-memory key and the presented session id. The store
has a fixed capacity, expires idle sessions, and every successful cookie-session
request returns `Set-Cookie` with the renewed bounded `Max-Age`. A changed token
fingerprint or expired server session denies use rather than renewing it.
Cookies use `HttpOnly`, `SameSite`, `Max-Age`, and `Secure` when TLS is indicated
by the operator front door.

## Redaction

Rendered pages, JSON responses, logs, audit rows, and errors must not include
daemon bearer tokens, web secrets, database passwords, forwarding secrets,
kubeconfig contents, cookie values, or raw stack traces.

## Authorization

The web adapter submits daemon command envelopes with a fingerprinted operator
attribution and correlation id. Responses set no-store, frame, MIME, referrer,
and restrictive content-security headers. The daemon remains the final
authorization boundary for every mutation. Unavailable actions render disabled
reasons instead of hidden or fake success.
