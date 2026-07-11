# Dynamic menus

## Purpose

This document records withdrawal of daemon-backed inventory surfaces. Only the
local docs browser and its navigation remain shipped.

## Status

implemented

## Phase model

The docs browser renders local bundled documents, directory entries, links,
search, pagination, Back, and Close. It has no daemon request phase, grant
snapshot, stale daemon cache, or mutation row.

## Stale policy

Malformed local metadata or an unavailable bundled document leaves the session
safe and gives localized diagnostics; it never invents data or an action.

## Domain surfaces

Travel, economy, profile, achievement, server, admin, claim, and temporary
adventure menus are withdrawn. Their daemon-capable Java bindings and effects
must not be registered or packaged until trusted identity/session attestation is
implemented.

## Diagnostics

The local docs binding distinguishes an empty search from malformed local
content. It never converts a daemon failure into a local menu state.

## Verification

Tests cover local docs navigation, metadata validation, pagination, and safe
failure behavior. They do not prove a withdrawn daemon menu.
