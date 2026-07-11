# Product journeys

## Purpose

This index connects player and operator goals to their semantic owner and
truthful surface state.

## Status

implemented

## Player journeys

| Goal | Current entry | Owner | State boundary |
| --- | --- | --- | --- |
| Enter and choose a server | Proxy, `/menu`, server list | Network | Unready servers stay disabled. |
| Learn what to do | `/docs` now; curated Help target | Player help | Local content has true empty/recovery states. |
| Set preferences and identity | `/lang`, Action Bar, UUID profile | I18n, identity | Cached visibility is never authorization. |
| Travel and settle | homes, RTP, claims | Travel, Claims | No charge or transfer on unsafe/unready state. |
| Earn and use points | shop, exchange, achievements | Economy | Delivery failure follows refund rules. |
| Claim timed rewards | kits, daily, vote links | Rewards | Stale data disables claim. |
| Play and communicate | party, mail, report | Social | Empty is distinct from unavailable. |
| Join an expedition | catalog and confirmation | Adventures | Start/transfer failure refunds when documented. |

## Operator journeys

| Goal | Current entry | Owner | State boundary |
| --- | --- | --- | --- |
| Manage runtime | CLI, `/lkjmc`, Admin menu | Admin, Network | Daemon authorization and diagnostics are final. |
| Moderate safely | reports and moderation commands | Social | Failed action is not a sanction. |
| Announce | authorized command adapter | Announcements | No automatic replay after uncertain send. |
| Start Discord link | Minecraft command | Discord | Discord completion is withdrawn pending trusted policy. |

## Shared state contract

Every dynamic surface declares loading, true empty, retry, and recovery behavior
in its owner document. Loading never blocks a Minecraft scheduler. Empty is a
valid successful result. Retry is explicit unless an idempotent owner flow says
otherwise. Recovery uses last good read-only data or a typed diagnostic and
never fabricates a mutation result.

## Evidence boundary

Rows labelled current are supported by repository source and checks named in the
coverage ledger. Target rows require implementation evidence. External outcomes,
such as vote-site verification, Discord availability, or a live server run, are
not implied by this index.
