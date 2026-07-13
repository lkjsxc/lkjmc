# Discord

## Purpose

This area owns the community and operator Discord adapter product contract.


## Status

partial

Missing: a trusted interaction policy with server-verified roles, replay storage,
rate limits, and server-side confirmation for every action.

## Table of contents

- [Bot service](bot-service.md)
- [Linking](linking.md)
- [Security](security.md)

## Contract

Discord is not a source of product truth. Its slash-command and interaction
surfaces are withdrawn: the service can only send an empty command registration
payload to remove prior `/lkjmc` commands. `interactionBind` is refused before
any listener or Discord REST work; missing credentials also produce a diagnostic.

## Outcome, journey, and evidence boundary

No Discord user action reaches the process or delegates to the daemon. No
signature, mapped role, or request-body principal is authorization proof because
no interaction listener is executable. Local tests cover bind withdrawal. The
guarded Discord lane can remove prior registrations with real credentials; it is
not a proof of an action surface or interaction service.
