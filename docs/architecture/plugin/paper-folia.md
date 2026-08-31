# Paper and Folia plugin

## Purpose

This document defines the supported Paper/Folia menu and readiness boundary.

## Status

implemented, not player-accepted

## Responsibilities

Paper registers `/menu` and `/docs` and maintains the hard-locked slot-8 menu
entrypoint. The compiled menu bundle contains exactly five local routes:

- `root`, with inert guidance for `/lkjmc status` and `/lkjmc server
  <instance-id>`;
- `docs-directory`, `docs-file`, `docs-links`, and `docs-search`, which read the
  curated documentation bundled inside the plugin jar.

The only clickable effects are local navigation, Back, and Close. Informational
items are inert. There is no menu mutation, confirmation, refresh, remote
snapshot subscription, daemon command dispatch, or generic action body.

One adapter owns each player's synchronous menu session. Every click must match
the active route, session, render revision, slot, and encoded item metadata.
Reopening, closing, quitting, changing locale, or disabling the plugin retires
the prior owner. Hot reload is unsupported: deployment replaces the full JVM,
and disable does not claim to close an already open inventory through a possibly
invalid Folia owner. Menu handlers perform no PostgreSQL, network, filesystem,
process, or worker wait.

After platform installation finishes, the common runtime starts one dedicated
heartbeat reporter. It reads no Bukkit entity or region state and performs its
bounded credential-file and loopback HTTP work on its own daemon thread. Every
ten seconds it sends an empty request under a three-second deadline using the
instance-bound Paper credential. A committed heartbeat means this plugin
lifecycle reached installation; stale data after 30 seconds fails readiness
closed.

## Authority

The menu has no remote authority. `/lkjmc` status and transfer effects remain
owned by Velocity; the backend menu only tells players which command to use.
The Paper credential has only `lkjmc.instance.heartbeat` and is not used by the
menu.

## Verification

The deterministic menu probe renders all five routes in production renderer
code, checks English and Japanese output, navigation, Back, explicit Close,
stale-render rejection, bundled-doc availability, synchronous owner retirement,
and jar absence of the removed snapshot and mutation classes. Heartbeat tests
cover exact loopback targeting, empty-body bearer requests, retry after outage,
secret redaction, and lifecycle shutdown.

These checks prove the candidate jar's local behavior, not a real player click.
Live menu acceptance remains blocked until an authorized online-mode player
opens and exercises the installed menu.
