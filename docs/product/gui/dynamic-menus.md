# Dynamic menus

## Purpose

This document records withdrawal of daemon-backed inventory surfaces. Only the
local docs browser and its navigation remain shipped.

## Status

implemented

## Phase model

The docs browser renders local bundled documents, directory entries, links,
search, pagination, Back, and Close. Java common may retain a revisioned
`menus/global` catalog view, but the shipped docs browser does not bind actions
or turn it into an interactive daemon surface.

## Stale policy

Malformed local metadata or an unavailable, expired, or reload-required view
leaves the session safe and gives localized diagnostics. It never invents data
or an action.

## Domain surfaces

Travel, economy, profile, achievement, server, admin, claim, and temporary
adventure actions remain withdrawn. Read-only catalog transport does not register
a route or effect; trusted identity/session attestation is still required.

## Diagnostics

The local docs binding distinguishes an empty search from malformed local
content. It never converts a daemon failure into a local menu state.

## Verification

Tests cover local docs navigation, metadata validation, pagination, and safe
failure behavior. They do not prove a withdrawn daemon menu.
