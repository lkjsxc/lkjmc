# Identity and onboarding

## Purpose

This document owns player identity boundaries and the first-session help
journey.

## Status

planned

## Current identity boundary

Minecraft UUID is the durable player identity used by profile, claim, economy,
and link records. Display names are presentation data. Stored
[account-link records](discord/linking.md) remain daemon-owned; admin principal
and role resolution is owned by [Admin](admin/README.md). No product document
may treat a display name, Discord role, or cached menu visibility as final
authorization.

## Target onboarding journey

After joining, a player should receive a localized route to curated help,
settings, and the next safe action without unsolicited mutation. This is target
work: the current `/docs` browser exposes the packaged documentation bundle and
is not yet a curated first-session experience.

## Surface states

Onboarding loads local help without daemon dependency. Missing or invalid help
content falls back to the documented help index; it must not show a fake task
completion. Identity-dependent data loads asynchronously, has a true empty
state, retries by explicit refresh, and recovers by disabling mutation while
showing a typed diagnostic.

## Evidence boundary

UUID-oriented daemon and store contracts support durable identity records. Java
identity adapters are withdrawn pending trusted identity/session attestation.
First-session completion, client reachability, and player comprehension require
future implementation and user research evidence.
