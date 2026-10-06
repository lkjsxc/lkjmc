# Player selection and party API continuation

Local branch: `work/direct-ux-20261007`.
Entry source: `dac9d7ed7d5871f6520c483064b89146dfe42803`.
Worktree: `/home/coder/workspace/lkjmc-direct-ux-20261007`.
This candidate is not pushed or deployed; current canonical Forgejo source has
not been established. A normal canonical-ref read was rejected before execution.

## Implemented behavior

Web player pickers load suggested identities on opening, before optional name
search. Candidates can be selected without typing. The list appears above the
search box and scrolls within a bounded height. Editing a query clears old
candidates, and late responses cannot replace a new query or chosen identity.
Submitting an unselected name still fails locally rather than sending a name as
an account UUID. Native combobox/keyboard behavior and field labels remain.

`GET /api/v1/players` with no query or an empty query returns up to 30 identities
ordered by same authenticated game server, shared party, shared team, accepted
friendship, then name/UUID. Same-server context requires live sessions, active
matching profiles/native identities, and the viewer's current access to that
server. It means the Minecraft server, not an individual native dimension.
Hidden activity does not supply a presence-based candidate, while already-known
team/party/friend identities can remain selectable without disclosing presence.
Blocks work in both directions, and merged, banned, or inactive accounts are
excluded. The single SQL snapshot returns only identity, name and rank: no
coordinates, server IDs, session IDs, or presence flags. Explicit name search
retains literal wildcard escaping, the 128-byte input bound, and a 30-result cap.

The preserved party draft is integrated with a stronger API boundary.
`party_create` accepts an omitted name and chooses a persisted editable default
from the creator's language. Existing membership is reused on duplicate clicks,
without creating orphan rooms. Legacy named creation remains supported.
`party_rename {party,name}` requires the expected party UUID and its current
leader; a website administrator receives no unrelated-party authority. A stale
form after leaving or changing party cannot rename the new party. Existing
membership, messages, invitations, room IDs and names are not migrated.

The native team collection now places all existing teams before Create team,
including later pages. Creation uses a writable book; existing teams keep their
banner. The protocol regression walks all pages of a 30-team fixture and checks
both the distinct icon and ordering.

## Explicit unfinished scope

The attempted direct-party Web change and proxy wake/MOTD change were rejected
before writing. The attempted generic-head/Help cleanup was also not applied.
Do not infer that the party screen or these gameplay features changed merely
because their API or test prerequisites now work.

Outstanding requirements include name-free party UI and later renaming, one
party screen, SMP-owned readiness/contribution controls, ordinary first-attempt
sleeping-backend routing, same-native-world/related-player game selectors and Tab,
MOTD, the nether-star first-slot launcher, category-head cleanup, Help removal,
root Return-to-SMP placement and the SMP teleport request/bring-here workflows.
The previous Files auto-preparation and full Hosting/file-row links at `dac9d7e`
are retained. Older active/work documents describe historical checkpoints.

## Verification and owned test environment

Party-specific real PostgreSQL tests passed: convergent creation, current-leader
rename, preservation, localization and stale-party rejection. New suggestion
tests cover ranking, private-access revocation, hidden presence, lease/profile
identity freshness, literal name search, bounded results and bilateral blocks.
The final Rust workspace suite passed all 111 tests: 18 host-agent, 9 Core
unit, 83 real-database integration and 1 system-message migration case. Evidence:
`.local/party-direct-evidence/final/rust-workspace.log`. All 28 source/CI policy
tests and 2 guarded protocol-database scope tests passed. Java dependency-byte,
shared-language, Paper capability checks and required plugin builds passed.

A 30-choice keyboard regression first failed because the active option stayed
outside the scroll viewport. The picker now scrolls only its bounded list and
preserves input focus. A separate empty-submit regression reproduced persistent
native custom-validity after clicking a suggestion; choosing a valid identity
now clears it. Both before-fix failures are retained under
`.local/party-direct-evidence/keyboard-before` and `/required-before`.
Final fixed-build Web acceptance completed with exit status zero:
`lkjmc-picker-final-acceptance-20261007-88c4`.
All 111 browser fixtures and all 27 state/API/language tests passed. The six
player-picker cases cover direct mouse choice, optional name search, raw-name
rejection, late responses, narrow screens, keyboard scrolling and native
required-field recovery. No skipped cases or retries were used. Shared catalog
validation passed: 1,793 IDs and 99 source files. The build retains the existing
large-chunk advisory; no bundle-size improvement is claimed.

The final exact Web build was also exercised against real Core: all four
browser journeys passed at `.local/ux/real-6406fa6b0ca4`. The earlier
`real-4952f7f06c06` browser run remains supplementary evidence. Final logs are in
`.local/party-direct-evidence/final/`, including local artifact byte hashes.
The acceptance harnesses exited successfully. No TCP listeners on
18091/25691/25692/25693 or UDP listener on 25693 remain. The separate test
database/worlds are retained for reproducibility; existing application data was
not used as the fixture.

This is still local acceptance, not canonical CI or deployment. Native tests
exercise private offline Java connections; licensed Java, actual Bedrock,
physical Incus guest operation and external production acceptance are not
established. Rejected Web/other-game changes are explicitly excluded above.

An independent task-owned loopback database was created using the established
development PostgreSQL service, without replacing its existing credentials or
application data. The guarded configuration is `.local/dev.json`; credentials
remain private. The existing artifacts/toolchains are read through local cache
links. Core and game processes are launched only by the established acceptance
harness and are stopped by it. Test accounts/worlds belong to this separate rig.

Real Core/browser acceptance completed all four journeys at
`.local/ux/real-4952f7f06c06`. The complete native network plus expedition crash/recovery lane passed at
`.local/ux/real-af19b03eb16a` with exit status zero and a `passed:true` receipt.
The network log confirms the new all-pages team-order/icon regression. Existing
queued sleep/wake/arrival, failure/timeout, combat, identity, inventory and
expedition recovery checks also passed. This does not establish a fix for the
still-unmodified ordinary pre-connect wake-only branch.
Offline Java protocol acceptance is not public licensed-Java, real Bedrock,
Incus-host or production acceptance.

## Publication boundary

Normal canonical Git access is still unverified; no substitute mirror push,
authentication change, production update or execution-restriction bypass is used.
Before release, refresh the canonical source, reconcile other work, run canonical
CI for the exact accepted source and follow the existing GitOps backup, rollback,
admission and live verification procedure. Local receipts do not establish a
production deployment.
