# Velocity plugin

## Purpose

This document defines the local-safe Velocity plugin contract.

## Status

implemented

## Shipped responsibilities

Velocity provides MOTD and tab-list presentation only. It may register listeners
needed solely for those local proxy views and does not call the daemon.

## Withdrawn responsibilities

`/lkjmc`, `/hub`, send and wake commands, transfer bridges, profile saves,
moderation checks, grant refresh, daemon server discovery, and dynamic server
registration are withdrawn pending trusted identity/session attestation. No
configuration or credential may re-enable them.

## Verification

Gradle tests cover the pure MOTD fallback and tab-list header/footer text.
They do not start a proxy or dispatch Velocity events. Containment checks inspect
sources, resources, and built jars for absent daemon clients, command
registrations, registries, and transfer or moderation bridges.

## Forwarding target

The default proxy uses online mode and modern player information forwarding with
a private `forwarding.secret` file. This runtime configuration does not give the
local-safe plugin daemon authority.
