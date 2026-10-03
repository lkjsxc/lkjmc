# Timeline and server tools

The primary source repository is https://github.com/lkjsxc/lkjmc. Forgejo
`lkjsxc/lkjmc-rebuild` receives the same main commit for isolated CI and release
artifacts. Forgejo gitops remains the canonical production deployment authority.

## Timeline

`GET /api/v1/timeline` accepts `kind=all|messages|events`, an optional `room` UUID,
`before` cursor, and `known` (comma-separated item IDs). `known_ids` is not part
of the contract. A page holds at most 50 items; clients retain and revalidate at
most 200 distinct items. The query is limited to 12 KiB and the response to
8 MiB. Cursors are bound to the account, room and filter, with a 512-byte limit.

Items are returned in increasing PostgreSQL timestamp and ordinal ID order,
including microseconds. `next_cursor` loads older items. Each item also carries
`before_cursor` so a bounded client can continue older history after trimming
its newer edge. `updates` refreshes known jobs and messages; `removed_ids`
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

## Stopped server files

Opening Files starts only the guest OS through `server_inspection {id,open:true}`.
Minecraft stays stopped. Admission allows 15 minutes for startup; a ready window
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

Migrations 0015 and 0016 are additive. Keep the verified previous release and
the canonical DB/world backup receipts. Recover through the existing gitops
deployment gate and saved plans; never copy state into another apply path.

## Verification boundaries

`scripts/ux_verify.py` starts its own Core and cleans up only that process. It
requires an explicitly named loopback protocol-test database. Browser integration
uses real Core/PostgreSQL; fixture browser tests cover controlled UI states.
Protocol tests use actual Paper/Velocity, signed departure and modern forwarding,
but offline Java clients. They do not establish real Java authentication,
Bedrock, an Incus VM wake, or production browser acceptance. Those require
separate runtime evidence. CI checks retain secret classification, fixed Java
dependency hashes, isolated PostgreSQL and the existing release provenance gate.
