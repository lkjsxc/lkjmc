# Player, Timeline, and server-tool contracts

Canonical source and deployment authority follow [README](../README.md):
Forgejo `lkjsxc/lkjmc` owns source and CI; Forgejo GitOps owns deployment.
Historical branch/release records do not identify current production.

## Direct navigation candidate

Authenticated pages use a sticky topmost breadcrumb hierarchy instead of a
visible duplicate page heading and explanatory subtitle. Native ancestor links,
a screen-reader heading, route focus, and browser titles remain. World cards and
team rows are whole native links; authorized lobby entries belong in Worlds.
Existing teams precede Create team. Opening a world is navigation, not a transfer.

Common People pages do not present SMP contribution controls. Timeline has no
standalone group-creation or reporting controls; existing conversation access,
private messaging and own-message deletion remain. Unavailable voice service is
not promoted. Stored reports, groups and contribution choices are preserved.
Files entry uses the automatic preparation described below. Player selection
and current direct-party/native changes follow the [local checkpoint](work/2026-10-07-party-game-ux.md).
This checkpoint is not a canonical or production release.

## Player selection and party naming

The player picker opens a bounded contextual list before optional name search.
`GET /api/v1/players` with no/empty `q` ranks current permitted game-server,
party, team and accepted-friend identities in one authorized snapshot. It never
returns coordinates or session/presence metadata. Hidden activity cannot supply
a presence-only suggestion. Blocks, current server access, session expiry and
profile identity are rechecked. Named lookup remains literal and bounded.

`party_create` accepts an omitted name and coalesces with an existing membership.
An editable default is chosen once in the creator's language. `party_rename`
requires `party` and `name`; only that party's current leader can rename it.
The expected ID rejects stale forms after a membership change. Existing data and
named creation are preserved. Web and native Party open the single membership
directly, create without a naming interruption, and allow the leader to edit its
name later. Ready/Leave/Transfer carry the expected party ID in new clients;
legacy missing IDs serialize unchanged. SMP Expedition preparation owns next-
expedition consent. Native team menus put Create team after existing teams,
using a writable book rather than another existing-team banner.

## Native travel and teleport decisions

Ordinary not-ready pre-connect requests from established game sessions submit
the existing durable join intent rather than leaving the player after waking.
Initial/recovery/tracked-connection fences remain; success requires actual arrival.

`teleport_request` accepts optional `here` (default false). Migration21 stores
the direction in its invitation, without reinterpreting existing requests.
Only the recipient can accept; both live SMP sessions, blocks and backend are
rechecked. The saved direction selects traveler and target, and newly accepted
jobs carry both session IDs. A stale session cannot be moved by an old approval.
Both directions use list-first selectors. A notification opens the exact request
decision; zero/one/multiple pending accept/deny cases are distinct. Both request
participants can read only their properly bound accepted teleport result, not
unrelated jobs, and receive terminal notifications after real effect settlement.
The reference plugin's RTP and stability-countdown features are not part of this
contract. Public/Bedrock/Incus acceptance is separate from offline native tests.

## Player reads

`GET /api/v1/view/play` returns permitted servers, a `play` context, three pending
invitations, and up to twelve friends with visible in-game presence. The context
contains `preferred_server_id`, `identity_ready`, and the current `game_session`.
The same session/identity context accompanies a world detail. A revoked private
destination is never selected from travel history. Pending requests, blocks,
hidden activity, and inaccessible private destinations do not reveal presence.

The public Minecraft ingress comes from `LKJMC_GAME_ADDRESS` in `/health/ready`
and `/api/v1/me`. An unconfigured address is null, not a guessed connection target.

`GET /api/v1/expeditions` returns at most 25 participant-visible journal rows and
`next_cursor`. Cursors are bounded and account-scoped, with timestamp/UUID ordering.
`GET /api/v1/expeditions/{id}` requires participation or ownership even for an
administrator. Core supplies preparation requirements, price, duration, and
available actions through `/api/v1/view/expedition`.

Expeditions belongs to the official survival server, lkjmcsmp. Its canonical
web routes are `/worlds/{serverId}/expeditions`, the `/journal` child (including
cursor pagination), and `/{expeditionId}` details. It is not a sibling of Worlds.
Every expedition route first resolves the authorized parent server and requires
its official-world kind before loading expedition data. The server remains the
route context for tabs, breadcrumbs, browser titles, and return links; the Core
APIs retain their participant authorization and existing gameplay behavior.

System-authored presentation content uses `{id, params}`. API errors retain their
machine `code` and carry this envelope in `error.message`. Generated asset titles
use nullable `title_message`; a null value means the ordinary title remains
verbatim. Raw logs and files are not translated.

## Teams and contributions

`GET /api/v1/view/social?section=teams` returns `teams[]` summaries and nullable
`contribution_team_id`. `GET /api/v1/teams/{id}` requires membership and returns
`team`, including effective actor permissions and up to 100 members. Member
pagination uses `after` and `members_next_after`; member flags remain raw so
role editors do not mistake effective leader authority for stored flags.

`team_leave` requires a team UUID. `team_contribution_set` takes a team UUID or
null. Memberships, room access, permissions, and economic owner IDs are scoped
independently. The first membership selects its team; additional memberships
never change selection. Membership removal clears a selected contribution team
without choosing another. Account linking retains the union of memberships,
combines overlapping flags, and preserves only the canonical account's selection.

Core snapshots `game_events.contribution_team_id` at first successful acceptance,
in the transaction that writes the event, progress, allowance, and ledger.
Validated duplicate events never resolve a new selection. Account, active-team,
and membership locks serialize acceptance with selection/removal. Personal
rewards do not depend on the contribution preference. Achievement projections
are grouped by personal/team owner. New teams start at zero land capacity;
existing achievements award capacity and market purchases consume it.

## Timeline

`GET /api/v1/timeline` accepts `kind=all|messages|events`, an optional `room` UUID,
`before` cursor, and `known` (comma-separated item IDs). `known_ids` is not part
of the contract. A page holds at most 50 items; clients retain and revalidate at
most 200 distinct items. The query is limited to 12 KiB and the response to
8 MiB. Cursors are bound to the account, room and filter, with a 512-byte limit.

Items are returned in increasing PostgreSQL timestamp and ordinal ID order,
including microseconds. `next_cursor` loads older items. Each item also carries
`before_cursor` so a bounded client can continue older history after trimming
its newer edge. The per-item cursor does not establish that another page exists.
A null `next_cursor` records the actual beginning before the client trims its
bounded window. Polling retains exhaustion; eviction of the beginning makes
discarded history loadable again. `updates` refreshes known jobs and messages; `removed_ids`
removes content that is no longer authorized. Deleted messages have empty
bodies. Current room membership and both directions of blocks apply to room
lists and message reads. Administrator status does not authorize unrelated DMs.

`GET /api/v1/rooms` provides up to 100 conversations, `rooms_next_cursor`,
`before`, `selected`, and up to 200 known room UUIDs. An authorized selected
conversation can appear outside the current page. Only `removed_room_ids`
revokes cached conversations; omission from a page does not imply revocation.
Room summaries hold at most 32 member details and a separate member count.

Messages continue through the authorized `message_send` command. Duplicate
message/job completion notifications, passive logs/files reads, and automatic
child startup/inspection cleanup jobs do not create duplicate Timeline activity.
Explicit file-session closes belong to the requesting account and include the
open/close operation in Timeline and completion details. Automatic expiry and
revocation cleanup remain excluded from activity.
Full operation details are read through the authorized job endpoint.

The combined Timeline has no composer. Selecting one conversation binds both
read history and message submission to that room. The current identity and
membership checks apply before accepting late responses or retaining drafts.

## Hosting state and uncertain outcomes

Managed-server and preset projections include `hosting`: limits, owned count
and storage, active reservations, remaining allowances, minimum allocations,
and `can_create` with a structured blocked reason. Ownership determines usage;
visibility or membership in someone else's server does not consume the viewer's
quota. Creation checks per-server RAM/CPU and total owned count/storage. Running
servers and active file guests share aggregate RAM/CPU/concurrent reservations.

Server projections include typed `status`: machine power, game availability,
observation freshness, active operation, and allowed actions with reason codes.
A running inspection VM does not establish a running or joinable Minecraft
server. Public operation summaries exclude commands, file contents, and private
results; `can_inspect` determines whether the viewer can read the full job.

Console delivery can end as `delivery_unknown`. A durable prepared receipt and
the exact command hash establish which attempt is uncertain. The original job
must never dispatch again automatically. Once the receipt is verified, its
maintenance ownership is released so logs and deliberate new actions can work.
Other uncertain physical effects retain their existing reconciliation rules.

`GET /api/v1/admin/operations` requires administrator access. `filter=active`
includes queued/leased/waiting; `failed` includes failed/delivery_unknown;
`history` includes all terminal states. Routine log/file reads are excluded from
both counts and rows. Pages contain 25 rows ordered by creation time and UUID;
512-byte cursors are bound to the account and filter. Lists render structured
messages and names; detailed payloads/results remain behind the job endpoint.

Two bounded passive-read leases can progress alongside the serial mutation
worker. Each read rechecks authorization, binding, and expiry. Reads do not wake
a sleeping VM, and restore excludes conflicting reads. Expired workers cannot
renew authority or begin another effect; physical receipts remain available for
the current lease holder to reconcile.

## Stopped server files

Entering Files requests the existing `server_inspection {id,open:true}` once
for an authorized custom server with fresh stopped state and no conflicting
operation. The listing starts after guest readiness; Minecraft stays stopped.
Close, expiry and revocation do not automatically reopen the session. Explicit
retry retains uncertain admission/job identity and its original open/close
intent. Another operator's private job remains private when reusing shared
readiness. Unsupported server kinds retain their existing refusal. Admission allows 15 minutes for startup; a ready window
expires at the earlier of that deadline and 10 minutes after readiness. Reads do
not renew it. Close Files, expiry and permission revocation perform cleanup.
Starting Minecraft transfers boot ownership, so an older inspection cannot stop
the running game. Inspection reserves the same owner/host resource allowances
as a running guest. Changes to reserved resources are frozen until handover.

Agent intent and boot ownership are durable. The host reconciles expiry even
when Core is unavailable. Guest mutation receipts, stale SHA-256 refusal,
rooted path resolution, link/mount/special-file rejection and size limits remain
in force. Platform-managed paths are the shared exact policy in
`ops/guest/managed-paths.json`; ordinary plugin directories and plugin configs
are available. Minecraft OP writes require a stopped custom Paper server and
one verified Java identity matching the active native UUID. Saved `ops.json`
changes apply on the next start; a missing change record means unknown, not a
claim about live OP. Hosting Member/Administrator and legacy operator roles are
separate from Minecraft OP. Owners are immutable members displayed once.

## Existing guest compatibility and recovery

The release contains both helper and managed-path policy hashes. New images and
trusted guests install both. Gitops records a saved tenant-helper plan under its
existing writer lock. Running guest OSes receive the reviewed public helper
pair and disabled autostart without restarting Minecraft or editing worlds,
runtime configuration, credentials or mutation receipts. A sleeping older guest
is upgraded through the closed deployment gate: checkpoint its disk, record a
durable boot intent, boot the guest for bounded maintenance, stop legacy game
autostart normally, verify both helper files, and restore its original stopped
power state. A stopped guest whose combined helper marker already matches is
left stopped. Interrupted maintenance retains the owned recovery intent.
Normal explicit Minecraft start also verifies the helper pair. Console polling
never boots or upgrades a guest.

Keep the verified previous release and the canonical DB/world backup receipts.
Before the player-first cutover, close new work admission and allow active
Expeditions and transfers to settle under the deployment gate. Snapshot and
independently restore/compare the database before migrating. Recover through the
existing GitOps gate and saved plans; never copy state into another apply path.

## Verification boundaries

`scripts/ux_verify.py` starts its own Core and cleans up only that process. It
requires an explicitly named loopback protocol-test database. Browser integration
uses real Core/PostgreSQL; fixture browser tests cover controlled UI states.
Protocol tests use actual Paper/Velocity, signed departure and modern forwarding,
but offline Java clients. They do not establish real Java authentication,
Bedrock, an Incus VM wake, or production browser acceptance. Those require
separate runtime evidence. CI checks retain secret classification, fixed Java
dependency hashes, isolated PostgreSQL and the existing release provenance gate.
