# Admin menus

## Purpose

This document records withdrawal of Java inventory admin menus.

## Status

implemented

## Current boundary

Admin inventory routes, server pickers, confirmation rows, config controls,
security controls, economy controls, moderation controls, audit views, and root
Admin entry are withdrawn pending trusted identity/session attestation. They are
not disabled fallbacks and must not be registered or packaged.

## Shipped controls

Attested CLI and web controls own admin operations. They require the documented
daemon authorization, exact target context, confirmation where applicable, and
redacted diagnostics. CLI and web behavior is documented by their owner
contracts; no Java inventory route mirrors it.

## Future rule

A future Java admin surface needs trusted authenticated player identity and
session attestation before it can ask for any grant or render any visibility.
Daemon authorization remains final; cached grants, platform permissions, and
`op` alone never authorize an action.
