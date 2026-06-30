# Web routes

## Purpose

This document defines the initial authenticated route contract.

## Route groups

- `GET /login` renders a local operator login form.
- `POST /login` creates an authenticated session from a configured web secret.
- `POST /logout` revokes the browser session.
- `GET /` renders status, doctor, and current adapter capability summaries.
- `GET /instances` renders `instance.list` with lifecycle state and actions.
- `POST /instances/{id}/start` delegates to `instance.start` or wake request.
- `POST /instances/{id}/stop` delegates to `instance.stop`.
- `POST /instances/{id}/restart` delegates to `instance.restart`.
- `GET /instances/{id}/logs` delegates to `instance.logs` with bounded lines.
- `GET /jars` renders `jar.list` and plugin asset inventory summaries.
- `GET /admin` renders `admin.role.list` and principal inspection data.
- `POST /admin/grants` delegates to `admin.grant.create`.
- `POST /admin/grants/{id}/revoke` delegates to `admin.grant.revoke`.
- `GET /audit` renders `audit.tail` and admin audit summaries.
- `GET /security/token` renders token rotation status and plan output.
- `POST /security/token/rotate` delegates to token rotation apply.
- `GET /wake` renders wake-and-join queue state.

## API routes

JSON routes live under `/api/` and use the same command mapping as HTML forms.
Mutating JSON requests require bearer or session authentication plus CSRF when a
cookie session is used.
