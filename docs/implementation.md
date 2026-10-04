# Implementation

lkjmc is a player-first Minecraft community. The current source is the Rust
Core/PostgreSQL application, React web client, Paper adapter, Velocity proxy,
and Incus host agent in this repository. Production infrastructure and deployment
are owned by Forgejo GitOps, not by a second state tree in this repository.

## Player journeys

Play resumes the current permitted world or a recent permitted destination.
Worlds separates discovery and play from Hosting administration. People contains
friendships, teams, and parties. Timeline combines personal activity; selecting
a conversation binds both its history and composer to that conversation.

The web shell uses a shared responsive design system. In-game menus adapt to the
current server: the SMP exposes Homes, Land, Market, Expeditions, and People.
Account settings and linking remain available from either surface.

Expeditions are temporary End worlds. Preparation reserves 1,000 coins and
12 Eyes of Ender. The three-hour lifetime begins at activation. Preparation
commits the ready participant roster; subsequent party changes do not revoke
that commitment. Entry, return, and re-entry use durable physical records.
Expiry recovers an established safe location or validated respawn fallback.
World deletion and refunds still require their physical receipts.

## Shared contracts

Core owns authorization, economic transactions, durable jobs, and player-facing
projections. Game adapters own Minecraft physical effects and durable recovery
records. Host operations retain the canonical management lock and saved-plan
checks. Structured server status distinguishes machine power, Minecraft state,
observation freshness, operations, and permitted actions.

A console command with verified uncertain delivery ends as `delivery_unknown`.
Its original attempt is never resent automatically. Independent bounded reads
allow diagnostics alongside a queued mutation, while restore excludes conflicting
reads. A lost worker lease fences subsequent physical and backup-control effects.

First-party messages use stable IDs and typed parameters, rendered in the selected
language at the receiving surface. English and Japanese catalogs are complete
and checked together. Generated achievement/rank/asset labels retain explicit
provenance; player-authored text and raw logs are preserved verbatim.

See [the API and recovery contracts](ux-contract.md),
[language rules](languages-and-ui.md), and [acceptance boundaries](acceptance.md).
Historical implementation notes remain in Git history; they are not current
production evidence.
