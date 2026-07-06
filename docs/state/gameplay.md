# Gameplay state

## Purpose

This file records shipped gameplay and data behavior.

## Status

implemented

## Player data

- Player identities are created before FK-dependent first-contact writes.
- Inventory snapshots are immutable payloads with metadata; current profile state
  points at a snapshot revision.
- Join/quit session records, save-on-quit, transfer acknowledgement, and
  wait-for-source-save workflows are implemented for plugin-mediated transfers.
- Homes, warps, settings, language, points, achievements, kits, daily rewards,
  votes, mail, reports, warnings, notes, moderation, and parties use PostgreSQL.
- Homes support daemon-backed get, list, set, and delete commands; the Paper
  home menu opens selected-home detail actions for teleport, update, and delete.

## Claims and economy

- Claim creation, achievement progression, and audit insertion commit together.
- Claim lookups back Paper protection decisions and the claim protocol smoke.
- Shops, exchanges, balances, achievement rewards, and vote rewards are durable.
- The default shop catalog is seeded by CLI, playable setup, and bootstrap apply;
  shop item lore shows price, balance, after-purchase balance, shortfall, and
  delivery state. Invalid Paper materials disable purchase rows, and failed
  scheduler-side item delivery calls a daemon refund by correlation id.

## Menus and transfers

- Menus are served by the document-driven engine: JSON route contracts, pure JVM
  kernel, pure bindings, and one Paper runtime adapter.
- Dynamic menu data is daemon-backed or local-source, localized from
  `config/locales/*.json`, paginated by the engine, and refreshed in place when
  size and session match.
- Achievements render as browser-style directory and detail routes with claim
  actions, Parent Directory, and Main Menu controls instead of category filters.
- The in-game docs browser uses engine routes for directories, files, links, and
  search instead of a separate inventory stack.
- Menu server rows emit real save-first profile transfers through Velocity, with
  localized sending and failure feedback on Paper.
- Locale catalogs have one committed source and are bundled into JVM jars at
  build time.

## Discord links

- Minecraft players receive one-time plaintext link codes only in the
  `player.link.begin` response.
- The daemon stores only link-code hashes with expiry and consumption state.
- Discord `link` and `unlink` slash commands delegate to durable daemon link
  commands; link-gated commands do not fake success for unlinked users.
