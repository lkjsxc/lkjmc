# Workspaces and multiple teams release candidate

Implementation base: `354b3d7f945d316e93ebfa625532fe2babd36e67`.
Integration branch: `work/workspaces-teams-20261005`.
Accepted scope: [workspaces and teams](2026-10-05-workspaces-and-teams.md).

Worlds now has quiet Expedition navigation. Hosting uses a compact inventory and
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

## Integration evidence

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
- The release gate includes Java team-menu policy checks, real Core browser
  journeys, and native game protocol coverage for simultaneous memberships,
  contribution selection, scoped roles, ownership choices, and team progress.

Integration checks and fixture screenshots do not establish deployment. The
final committed candidate must pass the canonical isolated CI; its exact source,
artifact manifest, and private acceptance receipt are the release proof.

## Coordinated release

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
