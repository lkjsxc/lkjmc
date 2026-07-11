# Homes

## Purpose

This contract owns durable named-home data and the withdrawn Java home boundary.

## Status

implemented

## Daemon commands

`player.home.get`, `player.home.list`, `player.home.set`, and
`player.home.delete` own durable homes. They return typed errors for not found,
invalid name, wrong owner, database unavailable, schema mismatch, target
availability, and permission denial.

## Java boundary

Paper `/sethome` and `/home`, Homes menus, scheduler teleports, cross-server
wake-and-join, and local player feedback are withdrawn pending trusted
identity/session attestation. A durable home record cannot move a player or
produce Java success copy.

## Verification

Store and daemon tests cover set, get, list, delete, and typed errors. Java
containment inspection proves home commands, menu bindings, and transfer adapters
are absent.
