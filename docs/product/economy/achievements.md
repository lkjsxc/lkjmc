# Achievements

## Purpose

This document owns player-visible achievement definitions, progress, rewards,
and browser navigation.

## Status

partial

Missing: browser-style directory routes, detail routes, Main Menu control,
Parent Directory control, and tests proving filter chips are absent.

## Definition fields

Each definition has id, category path, title key, description key, icon material,
criteria kind, threshold, hidden flag, repeatable flag, and reward entries.
Points are the default reward entry. Other reward entries must name a real
executor such as `mail`, `minecraft-item`, `kit`, `title`, `permission`, or a
restricted audited daemon-command executor.

## Browser root

The achievements route behaves like a docs browser, not a filter chip UI. The
root shows summary, Claimable Rewards, Getting Started, Economy, Travel, Claims,
Social, Adventure, Staff when visible, and Main Menu. Directory rows navigate to
children. Hidden achievements remain hidden until discovered, in progress,
claimable, or claimed unless an owner document defines inert mystery rows.

## Detail pages

Achievement detail pages show localized title, description, category path,
state, progress bar, numeric progress, criteria explanation, reward summary,
claim button when claimable, disabled reason when not claimable, Parent
Directory, and Main Menu.

## Reward state

Player-visible rows use locked, in progress, claimable, claimed, and
repeatable-ready when a definition supports repeatable windows. Claims are
explicit and idempotent by player, achievement id, reward id, and repeat window.

## Data shape

`player.achievements.list` supplies id, category path, title key, description
key, icon material, state, current progress, required progress,
hidden/discovered, claimable, reward summary, and disabled reason. Menus must not
invent fake claim actions.

## Verification

Pure menu tests cover root directories, detail pages, hidden behavior, Parent
Directory, Main Menu, and claim buttons. Tests assert that category filter chip
rows do not render.
