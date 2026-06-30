# Achievements

## Purpose

This document owns player-visible achievement definitions, progress, and rewards.

## Definition fields

Each achievement definition has id, category, title key, description key, icon
material, criteria kind, threshold, hidden flag, repeatable flag, and point
reward.

## Criteria kinds

- `first-login`
- `home-set`
- `claim-created`
- `shop-purchase`
- `exchange-commit`
- `kit-claim`
- `vote-reward`
- `mail-send`
- `party-create-or-join`
- `report-resolved`
- `daily-streak`
- `adventure-complete`
- `adventure-return`
- `block-exchange-total`
- `warp-use`

## Default set

Defaults include first join, first home, first claim, first shop purchase, first
exchange, first kit claim, first vote, first mail, first party, staff report
resolution, daily streaks, miner, builder, farmer, traveler, trader, social,
explorer, adventure completion, and safe adventure return achievements.

## Progress rules

Progress reducers are pure and idempotent by correlation id when one is
available. Rewards apply once. Hidden achievements stay hidden until progress
starts or the row is claimed. Listing shows claimed and unclaimed progress where
allowed by the definition.
