# Web control operations

## Purpose

This runbook owns setup and verification for the authenticated operator web
surface.

## Defaults

The daemon web listener binds to loopback unless the operator explicitly places
it behind a separate authenticated front door. The setup flow writes web secrets
to owner-limited files and prints paths or fingerprints, never secret bytes.

## Operator flow

1. Configure the web bind address, port, and secret file in JSON config.
2. Start or reload the daemon.
3. Open the private URL from the host or an authenticated tunnel.
4. Log in with the operator secret and confirm status and doctor data.
5. Perform mutations only through rendered forms or `/api/` calls that include
   CSRF protection or an explicit bearer token.

## Verification

`LKJMC_WEB_SMOKE=1 ./scripts/check-web-smoke.sh` may run an opt-in loopback
smoke. Without the flag, the script reports a skipped live check. Default tests
cover authentication denial, CSRF denial, daemon command delegation, redaction,
and audit behavior.
