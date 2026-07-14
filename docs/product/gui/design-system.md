# Design system

## Purpose

This contract defines shared menu presentation across all routes.

## Status

planned

## Roles

Closed roles are info, action, navigation, decoration, disabled, success, and
danger. Route JSON selects a Bukkit material from the compiled allowlist. The
renderer resolves title, item name, lore, state warning, and fallback copy from
the player locale.

Color may reinforce a role but never carries the only label. Every actionable
slot has readable non-color text. English and Japanese use matching placeholders
and equivalent neutral, warning, denial, and confirmation tone.

## State cues

Current rows use normal labels. Stale rows include an explicit stale label and
are inert. Unavailable rows state unavailable and suggest refresh or retry.
Permission and attestation denials state that the action was not performed.

## Single renderer

Root, dynamic, confirmation, and documentation routes use one renderer and one
metadata codec. No local docs renderer or hard-coded alternate menu is packaged.
