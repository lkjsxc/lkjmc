# Direct navigation: local Web candidate

## Status and identity

- Worktree: `/home/coder/workspace/lkjmc-direct-ux-20261007`.
- Branch: `work/direct-ux-20261007`.
- Entry source: `06a0c7e3f7c1542f324dde666d165752bbbf090a`.
- This is a partial local candidate, not a published or deployed release.
- Canonical source is Forgejo `lkjsxc/lkjmc`; GitOps remains the deployment authority.
  Refreshing origin failed because this executor had no usable HTTPS Git login.
  The cached `origin/main` is not current remote evidence.
- Core, Paper, Velocity, migrations, authentication, trusted CI, and deployment
  configuration have no changes in this candidate. Production was not modified.

## Implemented Web behavior

The authenticated shell uses one sticky top-of-page breadcrumb hierarchy, native
ancestor links, and a single current-page indicator. Redundant visible page
headings and their description paragraphs are removed. The screen-reader heading,
route focus handling, document title, skip link, and keyboard navigation remain.
Long names wrap, including Japanese at narrow viewport widths.

Play shows the actual world collection rather than a welcome/preferred-world
marketing hero. Worlds includes authorized lobby entries. World cards are native
links over their whole surface, including their padding. World details retain
explicit join actions without the promotional hero. Opening a card never submits
a transfer; the action still respects verified identity/session state, projected
permissions, duplicate-click suppression, and the durable arrival receipt.

Team rows are whole native links. Existing teams precede Create team. The common
People screens no longer expose SMP contribution selection, badges, or explanatory
marketing copy. Stored contribution selections, team members, permissions, rooms,
assets, and capacities are not changed. A new SMP-local contribution UI is not
implemented by this candidate.

Timeline removes standalone group creation, report selection/submission, and the
report-only conversation options menu. Existing conversations, private-message
creation, isolated drafts, and the author's message deletion remain. Existing
legacy groups are not deleted or orphaned. The unavailable-voice-service message
is omitted. Account/Administration report links and routes are removed; existing
report records and Core APIs are not deleted.

A Timeline item's `before_cursor` only identifies a position; it does not prove
that an older page exists. A null `next_cursor` records the real beginning before
trimming the 200-item retained window. Polling keeps that knowledge; eviction of
the beginning makes real discarded history loadable again. Empty/revoked windows
do not retain a phantom cursor. Microsecond ordering and known-ID authorization
revalidation remain unchanged.

## Verification

Final fixed-build command `lkjmc-ux-fixed-build-final-verification-20261007-512a`
completed with exit status 0. Logs are retained under `.local/direct-ux-evidence/`.

- Production-mode Web build passed. Vite still reports its existing large-chunk
  advisory; this candidate does not claim a bundle-size optimization.
- All 27 state/API/language tests passed (19 state, 3 API, 5 language), including
  five new exhausted-history/eviction/empty-window regressions.
- All 95 browser fixture tests passed, with no skipped cases or retries. Ten
  new direct-navigation cases cover whole-card/row padding clicks, keyboard
  navigation, authorized lobby display, exhaustion, retired controls and
  English/Japanese sticky hierarchy at 390/768/1440px. Existing responsive
  coverage still exercises six widths from 320 through 1440px.
- Shared-message validation passed: 1,793 message IDs, 98 source files examined.
- All four real-Core journeys are discovered and updated to the new UI, but
  were not executed. No new owned integration database/service was provisioned;
  the dedicated PostgreSQL port was already occupied and was left untouched.
- `git diff --check` passed. Screenshot artifacts are fixture output under
  `.local/browser-results/`, not live-production screenshots.

Browser fixture results are not a substitute for real-Core, Minecraft protocol,
canonical CI, rollback, or production checks. None of those lanes is claimed to
have passed for this candidate.

The real-Core journey source now follows team creation into its conversation
instead of the retired group/report flow. It checks reload persistence and own
message deletion. The multi-team journey seeds SMP contribution state through
the authenticated test API and verifies that it does not change common team
context, permissions, membership, or grouped progress. The lane still requires
its dedicated loopback service and real session, with no mock/skip fallback.

Earlier fixture failures referred to retired UI controls and were updated to
retain their underlying identity, receipt, draft, focus, permission, and scope
assertions. A subsequent run overlapped a development rebuild; its failed page
was the fixture's static-file-not-found response, before Console rendered. That
run is not acceptance evidence. Its artifacts are preserved in
`.local/direct-ux-evidence/pre-final-browser-results`; the final run builds first
and does not modify the served assets while the browser suite is running.

## Blocked and unfinished work

Execution safety checks rejected writes for automatic file-session admission,
party/Core changes, and the game-menu changes before applying them. Those writes
are not part of this candidate. No alternate credential route, production write,
or execution-restriction bypass was used. These are execution blockers, not a
missing product decision or a request for renewed deployment authorization.

The owner's outstanding requirements remain:

1. Files should automatically prepare its bounded inspection guest on entry,
   keep Minecraft stopped, load the listing, and expose reasons only on failure.
   Polling, explicit Close, expiry, and authorization loss must not reopen it.
   Hosting and file rows still need the same broad native-link interaction.
2. Party creation must not ask for a name. Keep one membership, open the current
   party directly, and allow the current leader to rename it later. Common party
   pages must not offer SMP expedition readiness; move that consent to SMP.
3. A first attempt to join a sleeping backend must retain the destination intent,
   show startup progress, and finish the actual transfer without a second click.
   Preserve session/generation fences, combat/access checks and observed-arrival
   completion; do not treat startup submission as success.
4. Tab and player pickers should show same-world players. Add relevant team/party
   candidates before optional manual-name search, without exposing hidden or
   inaccessible presence. Revalidate membership/session at command acceptance.
5. MOTD must be `A Minecraft Server`. The lobby menu token must be a nether star
   in the first hotbar slot, migrating only recognized PDC tokens and preserving
   ordinary items. Category icons must not be generic Steve heads. Existing
   teams precede a visually distinct Create team action.
6. Remove broken Help and the prominent root Return to SMP shortcut. Keep return
   available in its relevant expedition context. Add SMP TP request/bring-here,
   accept/deny and player-first selection, informed by `lkjsxc/lkjmcsmp`'s
   `docs/product/commands/teleport.md`. Notifications should open an explicit
   request decision, not accidentally accept; success requires actual arrival.
7. Continue reviewing all other page descriptions, duplicate headings, leftover
   decorative sections, single-party navigation and world-specific UI. The
   remaining offline-world join link and expedition presentation are not fixed.

## Confirmed source locations for continuation

`crates/core/src/hosting.rs` has the durable `server_join` command. The separate
route branch in `crates/core/src/services/game.rs` can call `wake_for_join` and
return `ready: false`. `plugins/proxy/.../LkjmcProxy.java`'s pre-connect path then
denies that connection; its durable `processJoin` path instead reports waiting,
connects after readiness and confirms observed arrival. The two paths are a
plausible cause of the reported first-attempt behavior, not a reproduced live fix.

`web/src/hostingFiles.tsx` still uses explicit `filesSession(true)` admission.
`crates/core/src/commands.rs` still requires `PartyCreate { name: String }` and
has no party rename command. `crates/core/src/queries.rs` still returns an empty
player list for an empty search. `plugins/paper/.../GameMenus.java` still has the
old launcher, party chooser, generic heads, Help and root return entry.

## Publication gate

Do not label this checkpoint as live. Before publication, reconcile with fresh
canonical source, complete applicable real-Core and game acceptance, obtain the
canonical CI artifact for the exact accepted source, and use the established
GitOps release/rollback/admission checks. Do not use cached refs, fixture results,
old production receipts, or a local Web build as deployment evidence.
