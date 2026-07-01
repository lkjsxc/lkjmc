# Achievements

## Purpose

This document owns player-visible achievement definitions, progress, and rewards.

## Definition fields

Each achievement definition has id, category, title key, description key, icon
material, criteria kind, threshold, hidden flag, repeatable flag, and reward
entries. Points are the default reward entry. `mail` is implemented as a durable
non-point executor. Other supported reward entry types must name a real executor,
such as `minecraft-item`, `kit`, `title`, `permission`, or a restricted audited
daemon-command executor.

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

## Menu display

The achievements root shows summary counts and category filters. Category rows
sort claimable achievements first, then in progress, completed, locked, and
hidden. Rows show localized title and description, icon material, category,
progress bar, numeric progress, state, reward summary, and exact disabled reason.

## Reward state

Player-visible achievement rows use this state machine: locked, in progress,
claimable, claimed, and repeatable-ready when a definition supports repeatable
windows. Rewards are claimed explicitly after criteria are complete. Claim
attempts are idempotent by player, achievement id, reward id, and repeat window.

## Progress rules

Progress reducers are pure and idempotent by correlation id when one is
available. Progress completion makes rewards claimable; reward delivery applies
once through real executors and durable claim rows. Hidden achievements stay
hidden until progress starts or the row is claimable or claimed. Listing shows
progress, reward summaries, claimability, disabled reasons, and claimed state
where allowed by the definition.
