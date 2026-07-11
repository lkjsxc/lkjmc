# Product contracts

## Purpose

This area owns user-visible network, adventures, command, Discord, GUI,
localization, travel, claim, and player sync contracts.


## Status

implemented

## Table of contents

- [Player help](player-help.md) is the target curated player-help surface.
- [Journeys](journeys.md) groups current and target user outcomes by surface.
- [Identity and onboarding](identity-onboarding.md) owns first-session boundaries.

## Product owners

- [Network](network/README.md), [Travel](travel/README.md), and [Claims](claims/README.md)
- [Economy](economy/README.md), [Rewards](rewards.md), and [Adventures](adventures/README.md)
- [Social and moderation](social.md), [Announcements](announcements.md), and [Discord](discord/README.md)
- [GUI](gui/README.md), [Commands](commands/README.md), and [I18n](i18n/README.md)
- [Admin](admin/README.md) and [Sync](sync/README.md)

## Product outcome

Players can enter a playable Java network, use localized self-service journeys,
and receive a truthful result rather than a decorative control. Operators can
observe and change durable network state through authorized surfaces.

## Journey and degraded behavior

A player enters the network, opens `/menu` or a documented command, reviews an
action's state, and receives completion, an exact disabled reason, or safe
failure copy. Adapters may use cached data for visibility; stale or unavailable
daemon data disables mutations and never invents balances, transfers, or success.

## Evidence boundary

This contract describes implemented repository behavior, not a claim that every
external integration is live. Deterministic checks cover documentation and
contract wiring; opt-in playable, Bedrock, Discord, and Kubernetes smokes prove
only their named prerequisites and runs. Product behavior must be real before it
is registered, and English and Japanese strings ship together.
