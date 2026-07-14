# Admin menus

## Purpose

This document defines the attestation-gated staff menu family.

## Status

implemented

## Visibility and data

Admin routes are part of the compiled catalog so topology is complete. Their
views require a current exact permission snapshot. Routing or other typed data
must also be current for active rows; stale and unavailable data is labelled and
inert.

## Actions

Instance, configuration, security, economy, moderation, audit, and announcement
operations are closed typed identifiers. Route documents contain no generic
daemon command or body. Every mutation requires its named current capability,
trusted session attestation, and a typed mutation port. This menu task provides
no such port, so actions deny truthfully rather than dispatching or claiming
success.

Attested CLI and web controls remain separate owner surfaces. A platform
permission or `op` alone never authorizes an inventory action.
