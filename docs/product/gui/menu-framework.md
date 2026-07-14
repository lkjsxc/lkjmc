# Menu framework

## Purpose

This document defines the single document-driven Paper menu framework.

## Status

implemented

## Pure core

The common JVM core loads the deterministic compiled route bundle into closed
records and enums. Rendering consumes a route, locale, typed snapshot view, and
session metadata, then returns immutable slots. It performs no Bukkit, network,
filesystem, database, process, clock, or credential work.

Views classify every dependency as `CURRENT`, `STALE`, or `UNAVAILABLE` and
carry its revision. Failures are closed: malformed bundle, unknown route,
missing parameter, stale render, stale response, busy session, unavailable
dependency, permission denied, unattested action, and unsupported operation.

## Actions

Navigation, Back, Refresh, Close, inert content, and typed mutation are the only
actions. Navigation and Back replace the open inventory. Close is explicit.
Mutation has no generic daemon body and cannot dispatch unless a current typed
permission snapshot grants its capability and an attestation verifier trusts
the exact session request. No mutation port is added by this menu task, so an
otherwise admitted mutation reports unsupported rather than success.

## Paper adapter

One Paper listener renders every route, including documentation. It correlates
player, route, session, request, render revision, slot, and action metadata.
Each session permits one pending request. Old-row responses and repeated clicks
are inert with localized chat feedback. Scheduler callbacks never wait on
transport or workers.

## Verification

Goldens cover high-traffic route states and all routes have deterministic render
coverage. The protocol-like harness drives the production adapter through open,
click, navigation, close, stale response, outage, locale, and repeated-click
sequences without claiming a live Minecraft client.
