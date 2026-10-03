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

The Web has separate Home, Servers, Friends, Chat, Teams, Parties, Manage servers,
Account and Administration pages. Child navigation uses stable hash paths such as
`#/servers/<uuid>/land` and `#/manage/servers/<uuid>/files`. Server lists have one
server per row. Main content uses one column at every width; Chat retains its
conversation list alongside the selected conversation on larger screens.

Home shows at most three invitations, unread notifications and recent actions,
with counts and links to dedicated histories. Histories use cursor pagination of
25 rows, with a separate unread filter. Team and party membership/settings, server
console/files/backups/members/settings, account privacy/linking/blocks/reports and
administration sections have separate pages and read payloads. Existing legacy
hash links redirect; the official server alias resolves to its actual UUID.
Authorization remains enforced by Core for every server subsection and command.

The interface uses a fixed dark palette, including form controls and native
selects. Shared surface, input, text, border, muted and link tokens prevent the
white-background input override. Mobile navigation supports Escape, focus
containment, a dismissible backdrop and inactive hidden links. Controls wrap long
labels, retain visible focus and fit widths down to 320px.

The game menu has separate Friends, Chat, Teams and Parties entries. Server tools
remain under SMP details. The bottom row is reserved for Back, pagination, Main
menu, page count and Close. Non-actions do not accept clicks; a consumed click
cannot run twice. Asynchronous page reads are superseded by newer navigation.
History is bounded and removed when the player leaves.
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
- Run Vite on loopback port 18194, then `node scripts/check_ui.mjs`: 588 page ×
  language × viewport cases (320/360/390/768/1024/1440), persisted language switches,
  anonymous English default with Japanese browser locale, and mobile navigation.
  The populated fixture checks input contrast (at least 4.5:1), one-column content,
  independent social pages, supported presets and JavaScript errors. This is a
  synthetic API fixture, not a production login.
- `node tests/game/network.mjs`: real private Velocity/Paper transport, menu
  navigation, English/Japanese preference changes, server travel and recovery.
  Public online authentication and real Bedrock clients are separate acceptance.

Design references: [W3C reflow guidance](https://www.w3.org/WAI/WCAG21/Understanding/reflow),
[W3C target size](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html),
[Paper inventory holders](https://docs.papermc.io/paper/dev/custom-inventory-holder/).

## Automatic server software

Production generates `/etc/lkjmc-server-presets.json` for Core from the canonical
GitOps `services/lkjmc/host-agent.json` presets. The authenticated preset endpoint
exposes software, version, Java and the minimum storage allocation. Creation validates this list before any
server/job is inserted. Custom JAR creation remains available within approved
hosting limits. Paper 1.21.11 uses Java 21 and pinned build 132; official 26.2
servers continue using Java 25. The official 26.2 adapter is not installed into a
personal 1.21.11 server.

The reviewed tenant VM image requires a root volume of at least 16 GiB. Production
creation rejects smaller allocations before inserting a server or a job, and the
creation page displays this minimum and starts at 16 GiB. Development fixtures can
still use smaller allocations without creating production VMs.

Incus raw queries carry `project` in the API URL and do not use the incompatible
`--project` CLI flag. Ordinary commands retain the explicit flag. Unsupported
presets are terminal failures only after both binding and daemon inventories
prove no VM effects; unprovisioned log requests return a clear terminal error.
Uncertain host mutations continue through the existing recovery workflow.
