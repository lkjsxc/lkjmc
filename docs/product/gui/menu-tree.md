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

`check-menus.py` validates repository-catalog metadata only. The containment
checker rejects withdrawn daemon sources and packaged artifacts; neither check
proves a Java route allowlist or rendered inventory behavior. They do not prove
a withdrawn inventory surface.
