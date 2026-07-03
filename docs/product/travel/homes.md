# Homes

## Purpose

This contract owns named player homes, `/home`, and home menu actions.

## Status

partial

Missing: `player.home.delete`, selected-home detail route, update/delete
confirmation routes, and exact Paper `/home` failure mapping for malformed or
wrong-server daemon responses.

## Commands

`player.home.get`, `player.home.list`, `player.home.set`, and
`player.home.delete` are the daemon commands for durable homes. They return
typed errors for not found, invalid name, wrong owner, database unavailable,
schema mismatch, target server unavailable, and permission denied.

`/sethome <name>` sets or overwrites a home at the current location. Overwriting
a named home from the menu requires confirmation. `/home <name>` validates the
name, sends the daemon request asynchronously, handles non-success, malformed,
missing, wrong-server, and deleted-home responses, re-enters the player
scheduler, uses async teleport when possible, and reports teleport failure
without raw stack traces.

## Menu flow

The Homes list shows real daemon homes and Set Home Here. Selecting a home opens
`home-detail` for that exact home. The detail route may show Teleport, Update to
Current Location, Delete, Back, and Main Menu.

Teleport does not require confirmation. Update and Delete require confirmations
with the selected home name, server id, and precondition metadata. Delete or
Update rows are not rendered unless the daemon and store support the action.

## Cross-server behavior

If a home targets another server, the adapter may transfer or wake-and-join only
when `instance.list` reports the target ready, registered, and joinable. Unknown
or unjoinable targets render exact disabled reasons.

## Verification

Store and daemon tests cover set, get, list, delete, and typed errors. Java menu
tests cover selected-home metadata. Paper tests cover `/home` failure modes,
teleport failure, and menu update/delete confirmations.
