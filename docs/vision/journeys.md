# End-to-end journeys

## Purpose

This document gives target journeys that connect outcomes to owner areas. They
are product direction, not a statement that every step is currently available.

## Operator journey

1. An operator installs or opens an existing deployment and identifies its
   configuration, credentials, and runtime prerequisites through Operations.
2. They inspect desired state, observations, diagnostics, and authorization
   through an approved CLI, Minecraft admin, or private web adapter.
3. They request a bounded change; Architecture validates and plans it, then an
   effect adapter records the resulting observation or diagnostic.
4. They verify the named deterministic or guarded check and use recovery
   guidance rather than retrying an uncertain mutation blindly.

Owners: [Operations](../operations/README.md),
[Architecture runtime](../architecture/runtime/README.md), and
[Product admin](../product/admin/README.md). The journey succeeds only with a
truthful result, not merely an accepted request.

## Player journey

1. A player enters through a configured Java path and sees only ready, joinable
   choices or an exact unavailable reason.
2. They use a command or menu to discover an activity, review its cost or
   consequence, and confirm an action when required.
3. The product executes a real daemon-backed action or retains the player in a
   safe state with localized failure or recovery feedback.
4. On transfer, reward, or purchase uncertainty, documented product policy
   preserves durable truth and states the next safe action.

Owners: [Product network](../product/network/README.md),
[GUI](../product/gui/README.md), [Economy](../product/economy/README.md),
and [Sync](../product/sync/README.md). Optional integrations remain optional;
the target is not evidence of live reachability.

## Agent journey

1. An agent reads state, owner contracts, and the relevant decision before
   proposing a bounded change.
2. It updates the owner document before behavior, preserves pure/effect
   boundaries, and does not substitute fake adapters or success states.
3. It runs the narrowest available checks, records exact results and skips, and
   updates state only when evidence supports a current-behavior claim.
4. A human can review the diff, evidence, risks, and one executable next step
   without recovering hidden assumptions from a conversation.

Owners: [Agent](../agent/README.md), [Architecture](../architecture/README.md),
[Repository](../repository/README.md), and [Operations verification](../operations/verification.md).

## Current boundary

For current journeys and their evidence, read [product journeys](../product/journeys.md)
and [state](../state/README.md). A target step may be selected for work only
when its owner contract, implementation, and appropriate proof are added.
