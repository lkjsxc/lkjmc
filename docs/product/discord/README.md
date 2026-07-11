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

Discord is not a source of product truth. Its slash-command surface is
withdrawn: the service can only validate signed pings and send an empty command
registration payload to remove prior `/lkjmc` commands. Missing credentials
disable that operation with an explicit diagnostic.

## Outcome, journey, and evidence boundary

No Discord user action delegates to the daemon. Signed non-ping interactions
receive an explicit withdrawn response, and mapped roles or request-body
principals are never authorization proof. Local tests cover withdrawal behavior.
The guarded Discord lane can remove prior registrations with real credentials;
it is not a proof of an action surface.
