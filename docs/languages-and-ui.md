# Languages and navigation

English is the initial language, including when a browser or Minecraft client is
configured for Japanese. `locales/languages.json` declares the supported languages
and date/number locales. English strings are stable message IDs; `locales/ja.json`
provides Japanese. Add a catalog and registry entry to add another language.
Rust validates saved selections against the registry; Vite and the shared Java
module package the same catalogs. Missing translations fall back to English.
User names, chat, server names and submitted evidence are never translated.

Anonymous Web visitors keep an explicit choice in local storage. After sign-in,
the account's `language` is authoritative. Both Web and the authenticated game
command path update this field under the existing CSRF/session/idempotency rules.
Paper and Velocity receive it through trusted profiles/projections. Account
linking retains the surviving account's preference, like its other settings.
Migration 0014 adds a column and format constraint without rewriting game data.

The Web keeps land, market and private End under the SMP server details. Server
hosting controls and account/privacy controls remain separate. Mobile navigation
supports Escape, focus containment, a dismissible backdrop and inactive hidden
links. Controls wrap long labels, retain visible focus and fit widths down to 320px.

The game menu groups server-specific tools under SMP details. Friends, invitations,
account linking, language and help are separate tasks. The bottom row is reserved
for Back, pagination, Main menu, page count and Close. Non-actions do not accept
clicks; a consumed click cannot run twice. Asynchronous page reads are superseded
by newer navigation. History is bounded and removed when the player leaves.
The configured lobby role offers a tagged compass only when hotbar slot 9 is empty
or already contains its own launcher. It does not replace ordinary items or add
launchers in the SMP.

## Verification

- `python3 scripts/dev.py test --locked --offline`: isolated PostgreSQL tests,
  including explicit default, persistence, account isolation, unsupported language
  rejection and CSRF. The existing cookie test explicitly fixes production mode
  so the development shell cannot alter its Secure-cookie assertion.
- `python3 -m unittest discover -s tests/ci -v`: catalog completeness and parameter
  parity for both frontends, plus the existing secret policy tests.
- Run Vite on loopback port 18194, then `node scripts/check_ui.mjs`: 120 page ×
  language × viewport cases (320/360/390/768/1024/1440), persisted language switches,
  anonymous English default with Japanese browser locale, and mobile navigation.
  This uses a synthetic API fixture, not a production login.
- `node tests/game/network.mjs`: real private Velocity/Paper transport, menu
  navigation, English/Japanese preference changes, server travel and recovery.
  Public online authentication and real Bedrock clients are separate acceptance.

Design references: [W3C reflow guidance](https://www.w3.org/WAI/WCAG21/Understanding/reflow),
[W3C target size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html),
[Paper inventory holders](https://docs.papermc.io/paper/dev/custom-inventory-holder/).
