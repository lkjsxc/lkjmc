# AGENTS.md

## Mission

Build and operate a real, small, reliable Minecraft network control plane.

The product is not complete because contracts, menus, handlers, tests, or documentation exist.

The product is complete only when a supported operator or player journey works against a real deployment and the result is observed honestly.

Prefer a smaller working system over a broader inert system.

## Scope

This file applies to the entire repository.

More specific `AGENTS.md` files may narrow local implementation details.

A nested file must not weaken the safety, evidence, deployment, or truthfulness rules here.

## Truth hierarchy

Use this order when sources disagree:

1. Fresh observations from the exact commit and environment under test.
2. Executable source and generated artifacts from that commit.
3. Tests that exercised the relevant real boundary.
4. Current concise owner documentation.
5. Historical documentation, archived plans, and prior handoffs.
6. Assumptions from conversation or memory.

Do not preserve a documentation claim merely because a checker accepts it.

Do not preserve a test merely because it is expensive or elaborate.

A skipped live check is not evidence of live behavior.

A mock, fake scheduler, fake player, fake process, fixture database, or protocol-shaped object is not a live deployment.

## Product priority

The default priority order is:

1. A clean installation into an unprivileged LXC system container.
2. A real Velocity proxy and at least two real backend servers reaching ready state.
3. Accurate CLI health, status, logs, start, stop, restart, backup, and restore.
4. Real `/lkjmc` command parsing and completion on Velocity.
5. A real player status query and server transfer.
6. A small Paper/Folia menu whose visible actions are real.
7. Recovery, update, rollback, security, and measured performance.
8. Additional gameplay or integration features only after the preceding journeys are stable.

Do not expand claims, economy, adventures, mail, Discord, Kubernetes, Bedrock, or public web administration merely because dormant code exists.

## Backward compatibility

Backward compatibility is not a requirement.

You may replace configuration schemas, wire protocols, database schemas, commands, package layouts, and deployment layouts.

You may delete historical migrations after taking a verified backup of any discovered deployment.

You may remove unused commands, routes, crates, tests, scripts, documents, and generated files.

Git history is the archive unless a current operational need justifies retaining an in-tree archive.

Do not add compatibility layers, dual writes, adapters, or migration shims without a current verified consumer.

Preserve unrelated host services and unrelated user data.

Before destructive changes to an existing lkjmc deployment, create and verify a restorable lkjmc-specific backup.

## Simplicity rules

Implement one end-to-end vertical slice at a time.

Prefer deletion over adding another abstraction around an inert path.

Prefer explicit typed endpoints over a generic command registry with mostly denied members.

Prefer a few real menu routes over a large generated route catalog.

Prefer one canonical representation over mirrored hand-maintained schemas.

Prefer compiler-enforced types over documentation topology checks.

Prefer standard tools and ordinary code over custom controllers and bespoke orchestration frameworks.

Do not create a framework for one implementation.

Do not add indirection solely to make a future possibility look supported.

Do not retain code solely because it has tests.

Do not impose a universal source-file line limit.

Split files when cohesion, reviewability, generated-code boundaries, or compiler performance justify it.

Do not split files to satisfy an arbitrary counter.

Every new persistent table, service, queue, protocol message, background task, and dependency requires a current consumer and an owner.

Every new abstraction must either replace duplicated current behavior or make the current vertical slice materially safer or simpler.

## Active repository state

Durable multi-turn state belongs in `docs/work/active.md`.

Keep that file concise.

It must contain:

- the current objective;
- the exact base and current commit;
- confirmed facts;
- decisions made;
- completed acceptance items;
- current failures or blockers;
- exact commands already run;
- the next executable step.

Update it at meaningful checkpoints, not after every edit.

Do not use an ignored `tmp/` controller as the only task authority.

Do not require a custom task-transition command before useful implementation work can continue.

Temporary evidence may live under `tmp/agent/` and must remain uncommitted.

## Read order

For a new or resumed task, read only what is needed in this order:

1. This file.
2. `docs/work/active.md`, if present.
3. `README.md`.
4. The exact source, tests, and concise owner document for the selected slice.
5. Relevant deployment evidence from the current environment.

Do not recursively read the whole documentation tree by default.

Use `rg`, `git grep`, file manifests, compiler errors, and targeted tests to locate authoritative code.

Read historical research only when it answers a specific unresolved question.

## Task selection

Follow an explicit user request first.

Otherwise continue the next executable item in `docs/work/active.md`.

If that item is obsolete, update the file with evidence and select the smallest item that advances a mandatory end-to-end journey.

Do not spend a turn only redesigning the backlog when executable product work is available.

A plan is a tool, not a completion artifact.

## Multi-turn execution

Assume work may span many turns and context windows.

At the beginning of a turn:

- inspect the branch, commit, status, and recent commits;
- read `docs/work/active.md`;
- confirm whether prior evidence still applies;
- continue from the next executable step.

Before a context boundary:

- leave the worktree coherent;
- commit a meaningful completed slice when possible;
- update `docs/work/active.md`;
- record exact test outcomes and remaining risk;
- name one next executable command or edit.

Do not encode essential state only in chat.

## Git discipline

Inspect `git status --short`, the current branch, and the base commit before editing.

Use the repository's current branch policy.

Do not force-push or rewrite shared history.

Do not discard unrelated user changes.

Use isolated worktrees when parallel agents will edit overlapping repository state or when the current checkout is not safe to modify.

Do not require isolated worktrees for a simple single-agent change when the current checkout is clean and intended for the task.

Commit coherent vertical slices.

A commit should leave the repository buildable or clearly document a narrowly bounded intentional intermediate state.

Use descriptive commit messages.

Include `Tested:` and `Not-tested:` trailers when useful.

Do not claim a check passed unless it ran against the commit or working tree being reported.

Push authenticated changes when the task and repository workflow require it.

Never report a push that did not occur.

## Architecture baseline

The default target architecture is:

- one Rust daemon owning orchestration and the private control API;
- one Rust CLI for operators;
- PostgreSQL as the only durable product database;
- one Velocity plugin for proxy identity, commands, and transfers;
- one Paper/Folia plugin for backend presentation and backend-owned effects;
- systemd inside one unprivileged LXC production container;
- local-process runtime management for the initial product;
- explicit versioned JSON messages with generated or shared typed bindings;
- no public daemon control listener.

Change this architecture only when measured evidence shows a simpler or more reliable alternative for the actual deployment.

## Rust rules

Keep domain decisions separate from process, filesystem, network, clock, and database effects.

Use explicit structs and enums for protocol and durable states.

Reject unknown or malformed external fields at the boundary.

Do not use `unwrap`, `expect`, `panic`, `todo`, or `unimplemented` in production paths.

Do not perform blocking PostgreSQL, filesystem, process, or network work on an async executor thread without an explicit blocking boundary.

Use bounded queues, bounded concurrency, deadlines, and cancellation where work can accumulate.

Do not add an async database framework solely for style.

Measure the current implementation and change the database client only when it improves correctness, simplicity, or observed performance.

Keep operation state transitions durable when retries or process crashes can create uncertainty.

Do not represent request acceptance as effect completion.

## Java rules

Target one explicitly pinned and live-tested Minecraft, Velocity, and Paper/Folia version set at a time.

Use Java 21 or the minimum version required by the selected supported platform, whichever is higher.

Keep shared protocol types in the common module only when both plugins consume them.

Velocity owns authenticated proxy player identity, `/lkjmc` commands, completion, and server transfer effects.

Paper/Folia owns inventory UI and backend-only platform effects.

Never block a Velocity event loop or Minecraft scheduler thread on database, network, filesystem, process, or worker completion.

All remote work must leave the callback immediately and return through the correct platform scheduler or continuation.

Folia entity, region, global, and asynchronous ownership must remain explicit.

Use platform APIs, not display-name parsing or raw packet assumptions, for product state.

Do not expose a menu action unless its real effect path exists.

## Trust and authentication

Document the practical threat model.

Processes inside the lkjmc service container are trusted components, but each plugin still receives a distinct least-privilege credential.

Use filesystem permissions and scoped credentials rather than speculative remote attestation.

Keep admin CLI access on a protected Unix socket or equivalent private local boundary.

Bind plugin HTTP access to loopback or a private container address only.

Never publish the daemon API directly to the internet.

Treat authorization header names and schemes case-insensitively where required by HTTP.

Treat credential bytes as case-sensitive opaque data.

Retain a regression test for mixed-case base64 credentials.

Never print, commit, upload, or include secrets in support bundles.

## Protocol rules

Maintain one canonical versioned protocol.

Generate Java bindings from the canonical source or use a source format consumed directly by both languages.

Do not maintain independent handwritten Rust, Java, JSON, and documentation schemas for the same message.

Keep the initial command set small.

Every operation must have:

- a real current caller;
- typed input and output;
- an authorization rule;
- a deadline;
- an idempotency or retry rule;
- a truthful failure result;
- a deterministic test;
- live evidence when the operation crosses a real external boundary.

Delete protocol members with no current caller.

## Database rules

PostgreSQL is the sole durable product truth unless a measured redesign explicitly replaces it.

Plugins do not connect directly to PostgreSQL.

Use transactions for invariants that span rows.

Use uniqueness and foreign keys for invariants the database can enforce directly.

Keep the initial schema narrow.

Do not recreate dormant economy, claim, adventure, mail, party, or moderation tables before a live journey requires them.

Fresh-schema tests are mandatory.

Backup and restore tests are mandatory before destructive production schema changes.

## Configuration rules

Use one versioned JSON configuration model for operator-edited configuration.

Keep generated runtime files separate from operator intent.

Validate the entire configuration before applying any effect.

Reject unknown fields unless an explicit extension map is required by a current consumer.

Pin exact runtime artifacts by version and digest.

Do not resolve `latest` during an ordinary daemon restart.

Do not write secrets into ordinary JSON configuration.

Use private secret files or an existing secret store.

## Asset and process rules

A playable claim requires real immutable Velocity and backend server artifacts.

Resolve artifacts through official or primary project APIs where possible.

Record source URL, project, version, build, size, and SHA-256.

Download to a temporary file, verify it, fsync it when durability matters, then publish atomically.

Never execute an unverified partial download.

Only expose the proxy listener publicly.

Keep backend listeners private to the container or loopback.

Track child process identity, start time, executable, working directory, and expected configuration.

Never kill an unrelated process because a PID was reused.

Startup success requires a relevant readiness signal, not only process creation.

Stop success requires confirmed exit.

## Home-server deployment rules

Live-discover the server before changing it.

Use the already authoritative LXD or Incus installation.

Do not install both.

Do not migrate between them as part of this project.

Use one unprivileged LXC system container for the production lkjmc service unless an existing verified lkjmc container should be repaired in place.

Do not use a privileged container, host networking, direct LAN bridge, host Docker socket, LXD/Incus socket mount, or broad host filesystem mount for ordinary operation.

Do not replace the host operating system.

Do not disrupt unrelated containers, routes, firewall rules, storage pools, reverse proxies, or services.

Snapshot or back up the existing lkjmc deployment before rebuilding it.

Prefer building release artifacts outside the production container.

Install only runtime dependencies into production when practical.

Use systemd inside the container.

Keep PostgreSQL private.

Expose only the required Minecraft proxy TCP port and an explicitly approved optional Bedrock UDP port.

Do not expose the LXD or Incus API.

Do not expose the daemon HTTP API.

Record exact container, image, profile, resource limits, network attachment, and exposure rules used.

## Third-party terms

Do not fabricate acceptance of the Minecraft EULA or another third-party agreement.

Use an existing explicit acceptance record when one is present.

If acceptance is absent, complete all code, packaging, and non-starting deployment work and report only the final start as blocked.

Do not download or redistribute artifacts in a manner prohibited by their terms.

## Testing strategy

Use four evidence tiers:

1. `check`: formatting, generation drift, static analysis, and focused unit tests.
2. `integration`: real PostgreSQL, real filesystem, and real child processes in disposable directories.
3. `network-smoke`: real Velocity and backend jars plus a real protocol client in a disposable network.
4. `production-smoke`: the deployed LXC service, public proxy path, restart, backup, and restore observations.

Run the narrowest relevant tier while iterating.

Run the full local tiers before a release or deployment checkpoint.

Do not rerun the most expensive suite after every small edit.

Cache dependencies and build outputs safely.

Tests must fail when the intended behavior is removed or inverted.

Prefer a few high-value integration tests over a large collection of source-text checkers.

Keep regression tests for:

- mixed-case base64 bearer credentials;
- `/lkjmc status` parsing;
- `/lkjmc server <id>` parsing;
- Brigadier or platform completion for `/lkjmc`;
- a real proxy-to-backend transfer;
- daemon restart while plugins reconnect;
- clean install into an empty root;
- repeated install or update;
- backup verification;
- restore into an empty database and data root.

## Live acceptance

Do not mark the project, a release, or a deployment complete unless the mandatory live journey ran.

At minimum, collect exact evidence for:

- daemon health from the production service user;
- PostgreSQL migration state;
- Velocity ready state;
- two backend ready states;
- proxy status ping;
- a real player or protocol client login in an authorized disposable or production lane;
- `/lkjmc` completion;
- `/lkjmc status` output;
- one successful server transfer;
- one failed transfer with truthful feedback;
- service restart and recovery;
- a verified backup;
- a restore drill.

A production deployment may be reported as deployed but not fully player-accepted when a real online-mode account is the only missing prerequisite.

State that distinction explicitly.

## Performance

Correctness and simplicity precede optimization.

Establish a repeatable baseline before changing architecture for speed.

Measure at least:

- daemon cold start;
- daemon idle RSS;
- daemon CPU at idle;
- health and network-status latency;
- transfer authorization latency;
- plugin queue depth and timeout count;
- server startup time;
- build time by tier;
- PostgreSQL connection usage;
- Minecraft tick or scheduler impact from lkjmc callbacks.

Do not optimize synthetic source checkers while the player path is unavailable.

Do not add caches without an invalidation rule and measured benefit.

## Documentation

Documentation follows and explains the real product.

Keep a small canonical set:

- `README.md` for installation and supported outcomes;
- `AGENTS.md` for agent operation;
- `docs/architecture.md` for the current architecture;
- `docs/protocol.md` for current wire messages and trust boundaries;
- `docs/operations.md` for install, backup, restore, update, and recovery;
- `docs/work/active.md` for current multi-turn state.

Additional documents require a current owner and clear reason.

Delete or condense stale research, duplicated contracts, historical task ledgers, and capability matrices when Git history already preserves them.

Do not update docs before code as a ceremonial barrier.

Update code, tests, and concise owner documentation in the same coherent slice.

## Agent and API cost control

Minimize context churn.

Do not repeatedly reread unchanged large files.

Use targeted searches and line ranges.

Keep tool output bounded and redirect verbose logs to `tmp/agent/`.

Record stable findings in `docs/work/active.md` so later turns do not rediscover them.

Use subagents only for independent, well-bounded investigations or tests.

Do not ask multiple agents to redesign the same architecture.

Do not generate large speculative documents before implementing the next vertical slice.

Consolidate repetitive verification entrypoints.

Prefer deterministic machine-readable summaries from long test runs.

Stop investigating when enough evidence exists to make a reversible implementation decision.

## Verification commands

Use repository-provided commands after the simplification work establishes them.

The desired stable interface is:

```sh
./dev check
./dev test
./dev integration
./dev network-smoke
./dev release
./dev deploy --target <discovered-target>
./dev production-smoke --target <discovered-target>
```

A `Makefile` or equivalent small wrapper is acceptable if it provides the same stable intent.

Until those commands exist, use the narrowest current Cargo, Gradle, Python, shell, Compose, and live commands and record them exactly.

## Completion and handoff

A handoff must state:

- the objective completed;
- the current commit and branch;
- files and behavior changed;
- deployment changes made;
- exact checks run and their exit status;
- exact live observations;
- skipped or blocked evidence;
- destructive actions and backups created;
- remaining risks;
- one next executable step.

Separate `implemented`, `tested`, `deployed`, and `observed by a real client`.

Do not collapse those states into one word.

Do not end with a vague invitation for more work.
