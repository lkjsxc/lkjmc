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
| Read network documentation | Documentation | `/menu` or `/docs` |
| Search an article | Documentation search | local docs search |
| Need help | Staff guidance | contact an operator outside Java commands |

## Surface states

Help is local bundled content: loading is brief and non-blocking, empty means no
curated article matches, retry repeats local lookup, and recovery returns to the
Help index. It does not expose daemon actions, Java command availability, or
state mutations.

## Locale ownership

Every title, summary, action hint, empty state, retry label, and recovery message
requires English and Japanese catalog keys owned by the feature owner. See
[I18n ownership](i18n/ownership.md).

## Evidence boundary

This is a target information architecture. Existing docs-browser tests prove the
current browser only; a curated route needs menu, locale, and player-journey
evidence before it can be called implemented.
