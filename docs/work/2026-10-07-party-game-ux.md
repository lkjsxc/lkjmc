# Direct party and game UX continuation

## Identity and release boundary

Worktree: `/home/coder/workspace/lkjmc-direct-ux-20261007`.
Branch: `work/direct-ux-20261007`.
Entry source: `14ddbe5f62709a36adee73aeafd5b13fa63d94a1`.
This checkpoint continues the prior Files, breadcrumbs, Timeline and player
selection work. It is a local candidate, not a production deployment.

A current `git fetch origin` reached Forgejo but failed because HTTPS Git had no
usable login. Git has no credential helper, Tea lists zero configured logins,
there is no Forgejo known-host entry and no SSH agent connection. No credentials
were printed, copied from another service, or worked around. Canonical main/CI
and migration-number compatibility are not established by the cached ref.
GitHub remains a mirror, not an alternative publication destination.

## Web interaction changes

Party creation submits the existing name-optional command immediately. The one
current party opens directly with its member list, current leader, Chat room,
leader-only invitation/name controls and a scoped Leave action. Its name is in
the top breadcrumb/browser title; there is no duplicate heading, selector, or
second Party/Members submenu. The former members URL renders the same workspace.
Name drafts survive ordinary refresh, but an invitation/name dialog is dismissed
when its party or current leader changes. Keyboard and narrow-screen layouts
remain supported. Visual review caught browser-default bullets in the initial
roster; the final list has aligned names/actions and separators instead.

Ready/Withdraw is offered only in the official SMP Expedition preparation view.
Its projection includes the current party ID. The common readiness route and
cross-link are removed. Costs, eligibility and roster remain visible without the
promotional Expedition hero, decorative world art or refresh instructions.
Collection entries have full native-link hit areas beneath separate actions.

The offline world action that merely navigated back to Play, the duplicate
Hosting server name, and the unavailable voice toolbar are removed. Existing
conversations, active voice connections and ordinary address-copy navigation are
preserved. SMP Meet up provides explicit go-to/bring-here choices, both using the
list-first player selector. Invitation rows identify the direction and expiry.

## Party request safety

`party_ready`, `party_leave` and `party_transfer` accept an optional expected
`party` UUID and lock/recheck the current active membership. New Web requests
include the expected ID. A stale action cannot affect a newly joined party.
Legacy clients may omit the field; serialization omits None to preserve their
original command/idempotency representation. Name changes still require an
explicit party ID and its current leader. Existing parties/messages are retained.

## Native menu and launcher

The lobby launcher is a tagged nether star in the first hotbar slot. Existing
tagged book/compass launchers are recognized. Ordinary items displaced from the
first slot move to the previous owned storage slot or an available storage slot;
a full ordinary inventory is not discarded or written into armor. Existing
swap/drop protection and launcher debounce remain.

Native Party creates without a name prompt and opens its current membership
directly. Members, later naming, chat, leader transfer and Leave are available
without a party chooser. Expedition consent is in SMP preparation. Help is
removed, and Return to SMP remains in Expedition context rather than the root.
People/Friends/member-management/invitation categories use non-player symbols;
heads represent actual players. Existing teams still precede the writable-book
Create action. Contribution controls are hidden outside the official SMP.
Standalone group-chat creation is retired without deleting existing groups.

Native target selectors load the authorized bounded Core candidate set first,
prioritize currently visible players in the same Bukkit world, then offer manual
name search. Core still applies block/access/session/profile constraints. The
initial candidate set is capped at 30 and is server/context-based; this is not an
unbounded roster or proof that every player in a very large world is listed.

Velocity's ping description is exactly `A Minecraft Server`, preserving its
other ping fields. This is verified on the actual Java status protocol; separate
Geyser/Bedrock MOTD and licensed/public connectivity have not been established.

## First-attempt sleeping-server travel

The ordinary pre-connect path previously woke the backend and denied the socket
without retaining a travel intent. A not-ready ordinary connection from an
established session now submits the existing durable join command and denies
only the premature socket. The queue owns readiness, cancellation, identity and
lease fences, departure saves, actual arrival and completion. Initial login,
recovery-lobby and already tracked attempt handling are unchanged.

The regression issues a real `/server lkjmc-<id>` command from a native client,
not an API call in its place. Before the fix, no durable job appeared and the
client stayed behind. After the fix, one command leads through wake/readiness to
actual Paper arrival with progress feedback. The other existing duplicate,
supersession, cancellation, reconnect, failure and timeout tests remain.

An unrelated fixture race sent a new destination after physical `/hub` arrival
but before its durable completion. The fixture now waits for the successful hub
receipt before its next move; the application admission guard was not weakened.

## SMP teleport requests

New additive migration `0021_teleport_direction.sql` records `teleport_here`,
false for all existing requests; its constraint restricts direction to teleport
invitations. The existing sender/recipient/pending index remains. Before release,
reconcile this migration number and permission with fresh canonical source and
the trusted GitOps schema gate; local tests are not migration approval.

The default request moves the requester to the accepting recipient. Bring-here
moves the accepting recipient to the requester. The saved invitation, not client
input at acceptance, determines direction. New requests supersede only the same
requester's pending pair; other requesters remain. Expiry, blocks, current SMP
presence and same backend are checked. Acceptance captures both live session IDs;
new jobs reject a later identity/session at the physical movement boundary.
Legacy queued jobs without the new fields retain their previous contract.

Native SMP exposes both directions plus pending requests. `/tpa` and `/tpahere`
open list-first selection; supplied names narrow verified candidates. `/tpaccept`
and `/tpdeny` show an explicit empty result, directly resolve one pending request,
or display a stable requester chooser for multiple requests. Native-created
request messages include the requester, direction and Core-recorded expiry. The
click action opens the recipient's exact request decision; it does not accept.

The existing actual teleport path still performs the movement. Completed/failed
chat and action-bar messages go to both participants only after the Core result
acknowledgement. Both participants also receive durable terminal notifications.
The result endpoint additionally admits only the counterpart of a correctly
bound accepted teleport invitation, with current blocks checked; unrelated job
results remain private and internal payloads are not exposed.

Reference: `lkjsxc/lkjmcsmp`, `docs/product/commands/teleport.md`, file SHA
`efe5b5e48c7502d23a41acf8afefbb1f60257f4a`. This implementation adopts explicit
direction/consent/request selection and actual-result feedback, not the entire
reference plugin. RTP, administrative `/tp`, its configurable five-second
movement-stability countdown and full expiry push signaling are not implemented
here. Web-created requests are durable invitations; immediate native chat pushes
for every Web action were not added. Do not claim complete reference parity.

## Validation evidence

Logs under `.local/party-ui-evidence/final/` and `.local/ux/real-*` identify actual
runs. The current terminal evidence is:

- Final formatted native network: `real-726d3ab395e9/game-protocol.log`, exit0.
  This includes the actual Tab `listed` wire flag and both TP directions. Its
  following failed Expedition stage is retained separately below, not hidden.
- Full Expedition crash/recovery after the fixture-only hostile-damage repair:
  `real-d38757c129ed`, exit0 and `result.json` with `expeditions:true,passed:true`.
  Original inventory counts, coin cost/refund, origin crash, entered crash,
  cancellation, offline expiry and safe fallback assertions all pass.
- Final real-Core/browser: `real-48126bc82b58`, all five journeys pass, including
  real name-free creation, scoped rename, reload persistence and scoped leave.
  Its `result.json` has `browser:true,passed:true`.
- At the final check, no owned Core/Paper/Velocity harness process or listener
  remained on TCP18091/25691/25692/25693 or UDP25693. Only the task-owned test
  database, worlds, logs and recovery receipts remain for investigation.

These are separate accepted network/Expedition/browser lanes on the recorded
local artifacts, not a claim that the earlier combined failed run passed.

The fixed Web build passed with all 120 controlled browser tests, including seven
new party cases and two teleport-direction/invitation cases. All 27 Web state,
API and language tests passed. Final Rust verification passed 115 tests (18 agent,
9 Core unit, 87 database/integration, 1 message-migration test). Shared catalogs
validate 1,805 IDs across 99 source files. All 28 source/CI and 26 guest-boundary
policy tests and both protocol-scope tests pass. No skipped cases or retry-only
acceptance are used for these summaries.

`artifact-hashes.txt` and `web-assets.sha256` retain exact local executable,
plugin and Web artifact hashes; they are not canonical release receipts.
`party-mobile-ja-final.png` and its compact WebP are the final controlled-fixture
screenshots. The corrected roster was visually inspected after verifying the
transferred compact image SHA-256, not inferred from markup or claimed as live
production. Screenshot names/player data are explicit fixture content.


Earlier actual before/after evidence is preserved:

- `real-637c869a474c`: original ordinary stopped-server attempt failed to retain
  a durable destination, before the pre-connect fix.
- `real-a03be6e13b05`: first-attempt travel passed, then a test compared a manual
  label's ASCII apostrophe to the catalog's typographic apostrophe. Corrected
  only the label assertion and retained the actual candidate-order check.
- `real-9dd09270a6dd`: later hub chaining raced durable arrival confirmation.
  The test now waits for that receipt rather than weakening the runtime fence.
- `real-4c2f1a039f18`: network and Expedition crash/recovery completed with exit0,
  including first ordinary connection, party/launcher changes, both TP directions,
  multiple requesters, explicit decisions and both-party actual completion.

- `real-726d3ab395e9`: final formatted native network passed, including the
  actual Tab listed flag. Its subsequent Expedition fixture failed at an
  inventory-count assertion. The corresponding raw owned-fixture log
  `adventure-8b1897.log` records `EndA8b1897 was slain by Zombie` while waiting
  for recovery. This is not silently counted as a passing combined run.
  The receipt fixture now applies and verifies Resistance V only on each
  generated offline test actor. Ordinary survival inventories, world rules,
  travel admission and explicit combat checks remain unchanged. Re-execution
  must establish the full original inventory/cost/crash assertions.

The final protocol assertions additionally inspect the wire update_listed flag,
not only Mineflayer's player profile map. This verifies that distant same-world
players are actually included in Tab. No broad reveal of isolated/holding players
or change to SpawnPolicy's isolation is introduced by this candidate.

## Remaining release acceptance

The final fixed-source lanes and owned-process cleanup are complete as recorded
above. Artifact hashes are retained locally and the source is saved separately
from publication. The real protocol fixture uses offline Java clients and
dedicated loopback data. It does
not establish licensed Java authentication, actual Bedrock behavior, public
connectivity, or physical Incus stopped-guest startup. Production data, existing
users, other worktrees and trusted deployment configuration remain unchanged.

No canonical CI artifact, current-main merge, rollback proof, push or production
release is claimed. A configured Forgejo login is required to resume that path.
Further UI simplification and large-roster/context refinements remain possible;
never describe this checkpoint as completing every conceivable UX issue.
