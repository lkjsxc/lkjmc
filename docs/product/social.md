# Social and moderation

## Purpose

This document owns player mail, parties, reports, and staff moderation outcomes.

## Status

implemented

## Ownership

- Mail owns private player-to-player messages and read state.
- Parties own membership, invitations, and party-scoped actions.
- Reports own player-submitted concerns and staff disposition.
- Moderation owns warnings, notes, bans, mutes, and their audit trail.

## Current journeys

Mail, party, report, and moderation commands delegate to daemon commands. A
player can read mail, send mail, create or join a party, and submit a report.
Permitted staff can inspect and resolve report or moderation state. These are
current repository contracts, not a claim that every community policy or offline
notification channel exists.

## Surface states

| Surface | Loading/retry | Empty | Recovery/degraded |
| --- | --- | --- | --- |
| Mail | Load asynchronously; retry refresh. | Show no messages. | Keep last read-only list; disable send/read mutations. |
| Party | Load asynchronously; retry refresh. | Explain no active party. | Disable invite/accept/leave until fresh state. |
| Reports | Load asynchronously; retry refresh. | Show no open reports. | Preserve stale list; disable disposition. |
| Moderation | Validate before mutation. | No matching history is valid. | Show typed diagnostic; never imply sanction. |

## Evidence boundary

Daemon registrations and store operations demonstrate supported paths. Paper
adapters are withdrawn pending trusted identity/session attestation. These paths
do not prove delivery to an external notification service, staff availability,
or a live moderation outcome.
