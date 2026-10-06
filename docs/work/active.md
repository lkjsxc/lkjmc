# Player selection and party API: local continuation

Current work: [player-selection checkpoint](2026-10-07-player-selection.md).
Entry source is `dac9d7ed7d5871f6520c483064b89146dfe42803`, including the
previous Files auto-preparation and full-row links. This continuation adds
list-first Web player suggestions, target-scoped party naming API, and native
team creation after the existing collection. Party Web and other game requests
remain unfinished. All 111 browser fixtures, 27 Web state/API/language tests,
111 Rust/real-database tests and four real-Core browser journeys passed. The
private native network plus expedition crash/recovery lane also passed. The
harnesses stopped their owned listeners. No push or production deployment has
occurred; canonical current-main/CI remain unverified.

---

# Direct navigation: partial local Web candidate

Current branch: `work/direct-ux-20261007`, based on
`06a0c7e3f7c1542f324dde666d165752bbbf090a`.
See [scope, verification and unfinished owner requirements](2026-10-07-direct-navigation.md).
This candidate is not published or deployed. Core/game/Files automatic admission
writes were blocked before application; canonical Git refresh needs a usable
Forgejo login. Existing production, other worktrees and their data are unchanged.
The fixed-build Web verification passed: 95 browser fixtures, 27 state/API/
language tests, and shared-message validation. The four real-Core journeys were
only discovered, not run. The release history below is not evidence for this
candidate.

---

# Gemini UX production candidate

User-requested design: `6114d5fb9603db0f6a7d4aa40fb49bef6246a1ea`.
Integration branch: `work/gemini-ux-production-20261006`.

Preserve Gemini's dark slate visual overhaul and the included SMP Expedition
navigation correction. Restore stylesheet behaviors found during release
review: reduced-motion preferences, visible keyboard skip navigation, wrapping
for long file-editor paths, the editor selector matching its actual markup, and
separation between Expedition participant names and their readiness labels.
Core, game, migration, and trusted workflow files are unchanged.

## Current release verification

- The production web build and all 22 state/API/language tests pass.
- After the execution permissions were restored, all 85 browser fixture cases
  passed, including responsive English/Japanese views at six widths, SMP parent
  authorization, file editing, multi-team actions, and operations. Playwright
  also discovers the four real-Core journeys required by canonical CI.
- Visual inspection found concatenated participant names and readiness labels
  in Expedition preparation; the restored roster layout separates these fields.
- All 14 affected preparation and responsive cases pass after that correction.
  Focused English/Japanese browser checks also confirm visible skip navigation,
  long file paths fitting 320px, monospace editor text, and reduced-motion support.
- Earlier Chromium and network permission failures are resolved. Production
  access uses the configured NetBird sign-in. Current host/remote checks,
  canonical CI, rollback proof, deployment, and live verification must be recorded
  by the release operator; historical receipts below are not current evidence.
- Refresh the baseline, run canonical branch
  CI, verify fresh rollback evidence and fence admissions, publish through the
  protected fast-forward path, verify exact-main CI artifacts, reconcile, and
  verify live assets and restored gate/timer state. Deployment is already
  authorized by the user; no new deployment approval is required.

## Included Expeditions correction

Implementation base: `575c9d021c64eb1bfefd23212802bb2d1b9a73dd`.
Integration branch: `work/smp-expeditions-20261006`.

Move Expeditions from the Worlds-level navigation into lkjmcsmp's own server
navigation. Overview, journal, detail, pagination, breadcrumbs, and browser titles
share the official server context. Direct links validate the authorized server
before loading expedition data. Retire the standalone `/expeditions` paths.
Core gameplay, participant authorization, database schema, and GitOps policy do
not change. Current verification and release results are recorded below when
available; the preceding release evidence is historical.

### Earlier local verification

- Production web build and all 22 state/API/language tests pass.
- Playwright discovers all 85 fixture and four real-Core browser cases.
- Browser regressions now cover nested overview/journal/details, official-server
  context, parent authorization and revocation, cursor navigation, return links,
  and English/Japanese layouts at six widths. Real-Core navigation discovers the
  official server before following its Expedition routes. Separate 403/404
  command regressions preserve immediate private-detail cleanup using the
  expedition ID while navigation retains the parent server ID.
- The 83-case browser fixture run could not start Chromium in the current
  executor (`sandbox_host_linux.cc:41`, `shutdown: Operation not permitted`).
  Every case stopped at browser launch; no application assertions or new visual
  verification completed. The test behavior has not been weakened or bypassed.
- Forgejo and the live site fail DNS resolution from this executor. Canonical
  CI, publication, and deployment have not been performed for this correction.

## Previous workspaces and multiple teams release

Implementation base: `354b3d7f945d316e93ebfa625532fe2babd36e67`.
Integration branch: `work/workspaces-teams-20261005`.
Accepted scope: [workspaces and teams](2026-10-05-workspaces-and-teams.md).

The preceding release added quiet Worlds-level Expedition navigation, superseded
by the server-owned hierarchy above. Hosting uses a compact inventory and
consistent server workspace, with authoritative creation allowances and explicit
runtime states. Files uses directory rows, breadcrumbs, URL navigation, and a
split desktop editor; mobile opens a dedicated editor. Drafts, SHA conflicts,
inspection sessions, staged uploads, and uncertain results retain their guards.

Accounts may belong to multiple teams simultaneously. Every team action and
economic owner is explicit. One optional contribution team receives automatic
shared progress; the first successful Core event acceptance fixes its recipient.
New teams earn their initial land capacity through achievements. Existing teams
retain capacity, balances, assets, roles, and room access. Account linking unions
memberships and access while preserving the canonical contribution selection.

Admin Operations replaces the attention summary with active, failed/uncertain,
and history views. Its authorized projection uses bounded cursor pagination and
structured messages rendered independently in English and Japanese. Web and
Paper expose the same team permissions and ownership choices.

## Previous release integration evidence

- All 105 local Rust tests passed: 18 host-agent, 9 Core unit, 77 integration,
  and 1 system-message test. The seven new team scenarios cover migration
  preservation, scoped permissions, contribution switching and retries,
  concurrent acceptance, transaction rollback, and account linking.
- All 81 browser fixture cases passed across the complete run and targeted
  corrections. Coverage includes seven team cases, four Operations cases,
  responsive English/Japanese views, URL history, stale deletion confirmations,
  draft isolation, exact creation payloads, and UTF-8 limits. An explicit editor
  label fixed the two failures found in the complete run; the four affected
  label, draft, and history checks then passed without weakening assertions.
- Java compilation, team-menu policy checks, and native fixture syntax passed.
  All 1,793 English/Japanese message contracts and 97 source files passed the
  locale checks. Visual review resolved all five initial layout findings.
- All four real Core browser journeys passed, including simultaneous team
  membership, contribution switching, team rooms, and grouped achievements.
  The achievements owner selector uses an explicit accessible label.
- The release gate includes Java team-menu policy checks, real Core browser
  journeys, and native game protocol coverage for simultaneous memberships,
  contribution selection, scoped roles, ownership choices, and team progress.

Integration checks and fixture screenshots do not establish deployment. The
final committed candidate must pass the canonical isolated CI; its exact source,
artifact manifest, and private acceptance receipt are the release proof.

## Previous coordinated release

Deploy migration 0020, Core, Web, and Paper together. The GitOps policy pins all
20 migration checksums and the existing trusted workflow. Publication uses the
normal candidate push check, protected fast-forward, and a separate normal main
push check for that exact commit. Deploy only the artifact verified from main.

Fresh canonical host/storage identity and capacity checks, an admission fence,
and verified recoverable database/world/configuration backups precede the
cutover. The prior deployed baseline is application `354b3d7` with 19 migrations
and GitOps `d820879`. Restore the reconcile timer after live verification.

Licensed Java, real Bedrock/console devices, voice, external WAN, and full
production recovery remain distinct acceptance boundaries in
[acceptance.md](../acceptance.md).
