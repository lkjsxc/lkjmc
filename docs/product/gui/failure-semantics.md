# Failure semantics

## Purpose

This document defines safe failure behavior for the local documentation menu.

## Status

implemented

## Local failures

An absent bundled path opens local search. An unknown click value, empty slot, or
missing item is inert. A malformed local document bundle prevents the local
surface from loading; it does not expose a host path, call the daemon, or
fabricate a player-visible result.

## Safe interaction

Only the local Close action closes an open documentation inventory. Navigation
and page changes preserve the inventory. Token repair cancels token movement and
restores its local state without reading a player setting or reporting a daemon
failure.

## Never allowed

The local surface must not emit raw error text, secrets, URLs, JSON, fake
success, hidden mutation attempts, or a fallback daemon action. Withdrawn
routes are absent rather than represented by a disabled mutation.

## Verification

Local menu checks and JVM containment prove bounded local behavior; they do not
prove a daemon-backed failure state.
