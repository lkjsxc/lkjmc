# Velocity plugin

## Purpose

This document defines the local-safe Velocity plugin contract.

## Status

implemented

## Shipped responsibilities

Velocity provides MOTD and tab-list presentation and owns one Java-common
read-only sync coordinator for plugin lifecycle. Listeners may read immutable
revisioned views; they never wait for HTTP and Velocity owns no poll loop.

## Withdrawn responsibilities

`/lkjmc`, `/hub`, send and wake commands, transfer bridges, profile saves or
application, moderation decisions, and dynamic server registration remain
withdrawn pending trusted identity/session attestation. Cached routing and grant
views are not proxy authority.

## Verification

Gradle and HTTP harness tests cover presentation fallback, one coordinator,
submit-return listener behavior, and clean disable. Containment inspects source,
resources, and jars for duplicate pollers, command registrations, mutation,
transfer, profile application, and dynamic registry bridges.

## Forwarding target

The default proxy uses online mode and modern player information forwarding with
a private `forwarding.secret` file. This runtime configuration does not give the
local-safe plugin daemon authority.
