# Player help

## Purpose

This document defines the curated, player-safe help surface.

## Status

planned

## Current and target

Current `/docs` opens the packaged documentation browser described by
[GUI docs browser](gui/docs-browser.md). The target Help route is a curated
player index, not an unrestricted filesystem browser and not an operator manual.
It must not be registered or advertised until its routes and locale keys exist.

## Curated journeys

| Need | Help destination | Safe next action |
| --- | --- | --- |
| Start playing | Network and settings | `/menu`, language, Action Bar |
| Move safely | Travel | homes or free overworld RTP |
| Protect a place | Claims | claim current chunk |
| Earn and spend | Economy | points, shop, exchange |
| Play together | Social | party, mail, report |
| Need help | Staff guidance | report, not privileged controls |

## Surface states

Help is local bundled content: loading is brief and non-blocking, empty means no
curated article matches, retry repeats local lookup, and recovery returns to the
Help index. Links to daemon-backed actions show their live availability and
exact disabled reason; Help never claims an action succeeded.

## Locale ownership

Every title, summary, action hint, empty state, retry label, and recovery message
requires English and Japanese catalog keys owned by the feature owner. See
[I18n ownership](i18n/ownership.md).

## Evidence boundary

This is a target information architecture. Existing docs-browser tests prove the
current browser only; a curated route needs menu, locale, and player-journey
evidence before it can be called implemented.
