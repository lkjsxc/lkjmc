# Web routes

## Purpose

This document defines the initial authenticated route contract.


## Status

implemented

## Route groups

- `GET /web/login` renders a local operator login form.
- `POST /web/login` creates a browser session from the daemon HTTP token.
- `POST /web/logout` revokes the browser session.
- `GET /web` renders status and doctor summaries.
- `GET /web/instances` renders `instance.list` with lifecycle state.
- `POST /web/instances/{id}/start` delegates to `instance.start`.
- `POST /web/instances/{id}/stop` delegates to `instance.stop`.
- `POST /web/instances/{id}/restart` delegates to `instance.restart`.
- `GET /web/audit` renders `audit.tail` summaries.
- `GET /web/security/token` renders token rotation status and plan output.
- `POST /web/security/token/rotate` delegates to token rotation apply.
- Planned: `GET /web/observability` renders bounded health, metrics, and event summaries.
- Planned: `POST /web/support-bundle` creates a bounded private bundle and renders only its manifest.

## API routes

JSON routes live under `/web/api/` and use the same command mapping as HTML
forms. Mutating JSON requests require bearer or session authentication plus CSRF
when a cookie session is used.

Every `/web` request enters the daemon's shared eight-lease admission before
login verification, credential lookup, denial audit, session work, rendering, or
command dispatch. It has the same eight-second whole-response deadline as a
command request. Saturation returns non-success before any web blocking work;
shutdown closes admission and waits for already admitted web work to exit. A
web deadline returns HTTP 408 JSON with `command.deadline_exceeded`; it never
falls back to a plaintext timeout. The command endpoint retains its HTTP 200
response-envelope contract while carrying the same non-success code.
