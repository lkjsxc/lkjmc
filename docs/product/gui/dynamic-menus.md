# Dynamic menus

## Purpose

This document owns daemon-backed dynamic inventory surfaces.

## Status

implemented

Implemented: bounded stale-data cache on all routes, selected-home detail routes,
achievement browser routes, profile-aware random teleport routes, and typed
daemon diagnostics on every adapter branch.

## Data policy

Dynamic routes first render loading, then loaded data, true empty state,
permission state, stale loaded data with warning, or typed diagnostic. Empty
lists are shown only when the daemon returned a valid empty list. Transient
failure may reuse recent successful data for the same player and route.
Loading, loaded, stale, and diagnostic replacement preserve the route stack and
session.

## Travel

Travel uses daemon-backed homes, warps, teleport requests, and random-teleport
quotes. Selecting a home opens a selected-home detail screen. Teleport is direct
and does not require confirmation. Updating a home to the current location and
deleting a home require confirmation and real daemon commands. Random Teleport
uses profile quotes: free overworld, paid Nether, and paid End.

## Economy

Economy uses points, shop, adventure catalog, kits, votes, and daily reward
data. Shop rows show balance, price, post-purchase balance, category, delivery,
and disabled reason. The economy route does not keep a click-only balance row.
Refund-safe item purchase may be direct. Adventure purchase remains confirmed.

## Profile and achievements

Profile summarizes balance and progression. Achievements render a browser-like
tree of directories and details rather than filter chips. Claim buttons call the
real daemon reward command only when claimable.

## Server routes

Public and admin server lists show all known instances with desired state,
observed state, readiness, health, connect address, proxy registration, player
count, joinable flag, and exact disabled reason. Enabled transfer rows only
appear when Velocity can transfer to the ready registered backend.

## Diagnostics

Menu diagnostics classify missing config, invalid config, token missing, token
unreadable, auth rejected, HTTP connect failure, HTTP timeout, command unknown,
command failed, database not configured, database unavailable, schema mismatch,
and permission denial. Player copy never includes secrets, raw URLs, raw JSON,
or stack traces.
