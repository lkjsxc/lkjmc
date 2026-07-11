# Rewards and voting

## Purpose

This document owns kits, daily rewards, and vote-link reward journeys.

## Status

implemented

## Ownership

Kits own configured reward claims and cooldowns. Daily owns one daily claim
state. Votes own configured vote links and operator-recorded rewards. Achievement
rewards remain owned by [Economy](economy/achievements.md), not this document.

## Current journeys

A player loads kit, daily, or vote state before acting. A kit or daily claim
returns a daemon result; a vote view supplies configured links. Operators record
vote rewards through daemon-backed administration. This is current repository
behavior; it does not claim that an external voting site has verified a vote.

## Surface states

| Surface | Loading/retry | Empty | Recovery/degraded |
| --- | --- | --- | --- |
| Kits | Load asynchronously; retry refresh. | State that no kits are available. | Preserve stale rows; disable claims. |
| Daily | Load asynchronously; retry refresh. | Never invent readiness. | Disable claim with typed diagnostic. |
| Votes | Load asynchronously; retry refresh. | State that no links are configured. | Preserve links only when cached; disable reward action. |

## Evidence boundary

Daemon command registrations, PostgreSQL store modules, and Paper adapters cover
repository paths. External vote attribution, site uptime, and reward policy
approval require external evidence and are not asserted here.
