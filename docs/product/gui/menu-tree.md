# Menu tree

## Purpose

This contract defines the shipped local documentation-menu hierarchy.

## Status

implemented

## Shipped routes

`/menu` and the hotbar token open the bundled documentation list. `/docs` opens
a bundled path or searches bundled content. Selecting a local document opens its
paged local content. Previous/next, Documentation, and Close are local controls.

## Withdrawn routes

Network, travel, claims, economy, social, profile, settings, admin, adventure,
confirmation, and all daemon-backed child routes are withdrawn pending trusted
identity/session attestation. No placeholder row or disabled mutation is
registered.

## Verification

Containment and menu checks prove the local route allowlist and absence of
daemon actions. They do not prove a withdrawn inventory surface.
