# Product contracts

## Purpose

This area owns user-visible network, adventure, command, Discord, GUI,
localization, travel, claim, and player-sync contracts.

## Status

implemented

## Table of contents

- [Player help](player-help.md) is the target curated player-help surface.
- [Journeys](journeys.md) records supported and unavailable entrypoints.
- [Identity and onboarding](identity-onboarding.md) owns first-session boundaries.

## Product owners

- [Network](network/README.md), [Travel](travel/README.md), and [Claims](claims/README.md)
- [Economy](economy/README.md), [Rewards](rewards.md), and [Adventures](adventures/README.md)
- [Social and moderation](social.md), [Announcements](announcements.md), and [Discord](discord/README.md)
- [GUI](gui/README.md), [Commands](commands/README.md), and [I18n](i18n/README.md)
- [Admin](admin/README.md) and [Sync](sync/README.md)

## Product outcome

Players can read bundled local documentation through Java presentation surfaces.
Operators can observe and change durable network state through authorized daemon,
CLI, and web surfaces. Java player mutations are unavailable pending trusted
identity/session attestation.

## Evidence boundary

This contract describes bounded repository behavior, not a claim that an
external integration is live. Deterministic checks cover contracts; guarded
checks prove only their named prerequisites and executed behavior.
