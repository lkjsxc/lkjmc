# Temporary instances

## Purpose

This document defines future daemon-managed short-lived Minecraft backends.

## Contract

Temporary instances are real instance records with unique ports, generated world
directories, strict maximum lifetime, autosuspend enabled, hidden normal server
listing, and cleanup policy. They are not plugin-local world tricks.

## Lifecycle

A service creates a session, creates a temporary Folia instance, starts it,
waits for readiness, registers it through Velocity, transfers participants, and
stops it on success, timeout, disconnect, or empty state. After retention, the
daemon deletes or archives the world directory according to configuration.

## Atomic service rule

Point deduction, session creation, and instance creation must commit in one
daemon-side transaction. If process start later fails, the daemon refunds
through the points ledger, marks the session failed, and audits the transition.

## Current boundary

No temporary adventure daemon commands or live purchase menu actions may be
registered until creation, transfer, stop, and cleanup work end to end.
