# Web control operations

## Purpose

This runbook owns setup and verification for the authenticated operator web
surface.


## Status

implemented

## Defaults

The daemon web listener binds to loopback unless the operator explicitly places
it behind a separate authenticated front door. The setup flow uses the daemon
HTTP token file as the local operator login secret and prints paths or
fingerprints, never secret bytes.

## Operator flow

1. Configure `daemonHttp.address` and `daemonHttp.tokenFile` in JSON config.
2. Start or reload the daemon.
3. Open the private `/web` URL from the host or an authenticated tunnel.
4. Log in with the operator token and confirm status and doctor data.
5. Perform mutations only through rendered forms or `/web/api/` calls that
   include CSRF protection or an explicit bearer token.

## Verification

`LKJMC_WEB_SMOKE=1 ./scripts/check-web-smoke.sh` may run an opt-in loopback
smoke. Without the flag, the script reports a skipped live check. Default tests
cover authentication denial, CSRF denial, daemon command delegation, redaction,
and audit behavior.
