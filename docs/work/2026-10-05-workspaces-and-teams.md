# Workspaces and multiple teams

Accepted implementation plan, based on application `354b3d7` and GitOps `d820879`.

- Remove the Worlds Expedition promotion; retain quiet Worlds/Expeditions navigation.
- Rebuild Hosting as a compact server inventory and consistent server workspace.
  Overview remains the landing view. Show truthful machine, game, and connection
  states, owner creation allowance, and running/file-guest resource reservations.
- Replace file button tiles with a semantic directory table, plain breadcrumbs,
  and a focused action toolbar. Use a split editor at 1024px and above and a
  dedicated editor below. Preserve sessions, authorization, SHA conflicts,
  staged uploads, dirty drafts, bounds, and uncertain-operation handling.
- Allow simultaneous team memberships. Team routes, actions, permissions,
  wallets, land ownership, and achievements carry explicit team/owner context.
- Select at most one contribution team for automatic shared rewards. Backfill
  existing memberships; select a first membership automatically, never switch
  for additional memberships, and clear selection when its membership ends.
  Persist attribution with the first successfully accepted Core game event;
  retries cannot redirect rewards. Personal rewards remain independent.
- New teams start with zero land capacity and earn capacity through existing
  achievements. Market land purchases consume capacity; they do not buy it.
  Preserve every existing team's assets and allowance.
- Account linking unions team memberships and room access, combines overlapping
  permissions, retains canonical contribution selection, and keeps the existing
  leader restriction. Parties remain unchanged.
- Replace admin attention counts with neutral Operations, with active,
  failed/uncertain, and terminal-history filters, administrator authorization,
  bounded cursor pagination, and localized structured messages.

Migration 0020 removes the account-only membership uniqueness constraint, adds
an account lookup index, an optional membership-constrained contribution
selection, and historical event attribution. Release database, Core, web, and
Paper together without compatibility adapters.

Validate transactional multi-team/reward behavior, identity linking, existing
file security/recovery contracts, quota and operation projections, real Core
browser flows, keyboard navigation, and separate English/Japanese rendering.
Review widths 320, 390, 768, 1024, and 1440px. Complete the canonical isolated CI
and coordinated deployment workflow with fresh backups/capacity verification.

This file records intent, not verification or deployment evidence.
