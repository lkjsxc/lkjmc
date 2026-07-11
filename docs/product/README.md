# Product contracts

## Purpose

This area owns user-visible network, adventures, command, Discord, GUI,
localization, travel, claim, and player sync contracts.


## Status

implemented

## Table of contents

- [Admin](admin/README.md)
- [Adventures](adventures/README.md)
- [Claims](claims/README.md)
- [Commands](commands/README.md)
- [Discord](discord/README.md)
- [Economy](economy/README.md)
- [GUI](gui/README.md)
- [I18n](i18n/README.md)
- [Network](network/README.md)
- [Sync](sync/README.md)
- [Travel](travel/README.md)

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
