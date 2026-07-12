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

## API routes

JSON routes live under `/web/api/` and use the same command mapping as HTML
forms. Mutating JSON requests require bearer or session authentication plus CSRF
when a cookie session is used.

Every `/web` request enters the daemon's shared eight-lease admission before
login verification, credential lookup, denial audit, session work, rendering, or
command dispatch. It has the same eight-second whole-response deadline as a
command request. Saturation returns non-success before any web blocking work;
shutdown closes admission and waits for already admitted web work to exit. A
web deadline returns HTTP 408 JSON with `command.deadline_exceeded`, matching the
TCP deadline contract; it never falls back to a plaintext timeout.
