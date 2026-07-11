# Locale ownership

## Purpose

This document assigns ownership for player-visible locale keys and states.

## Status

implemented

## Ownership rule

The product owner of a journey owns its English and Japanese keys, including
loading, empty, retry, disabled, recovery, confirmation, and success/failure
copy. `i18n` owns catalog format, fallback, strict rendering, parity checks, and
the shared text pipeline; it does not invent feature wording.

| Key family | Semantic owner |
| --- | --- |
| `achievement.*`, `achievements.*`, `points.*`, `shop.*`, `exchange.*` | Economy |
| `adventure.*` | Adventures |
| `announcement.*` | Announcements |
| `claim.*` | Claims |
| `command.*` | Commands |
| `docs.*` | Player help |
| `home.*`, `rtp.*`, `teleport.*`, `warp.*` | Travel |
| `kit.*`, `daily.*`, `vote.*` | Rewards |
| `language.*` | I18n |
| `mail.*`, `party.*`, `report.*`, `reports.*`, `moderation.*` | Social |
| `note.*`, `notes.*`, `warning.*`, `warnings.*` | Social |
| `actionbar.*`, `daemon.*`, `diagnostic.*`, `hotbar.*`, `hud.*`, `menu.*` | GUI |
| `server.*`, `velocity.*`, `wake.*` | Network |

Every prefix currently present in `config/locales/en.json` appears once above;
the listed owner is the only semantic owner for that family.

## Change and recovery rule

A feature change adds both language values before code references a key. A
missing key falls back through the documented language chain and exposes the key
only as the existing last-resort behavior; it must not be replaced with an
adapter-authored English sentence. Invalid catalogs fail build checks rather
than silently degrading player copy.

## Evidence boundary

`config/locales/*.json`, common catalog loading, and locale checks prove bundled
catalog integrity. They do not prove translation quality to a player; that needs
native-language review.
