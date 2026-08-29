# AGENTS.md

## 1. Purpose and scope

This file defines durable repository-wide policy for agents working in lkjmc. A more specific
`AGENTS.md` may add rules for its subtree, but it must not weaken repository-wide safety, evidence,
data, release, or deployment requirements.

lkjmc should remain a small, truthful, release-oriented Minecraft control plane that can be
understood and maintained by future AI coding agents, including weaker models. Optimize for real
operator and player outcomes, correctness, recoverability, architectural contraction, and proof at
the affected boundary. Do not optimize for apparent sophistication, number of features, diff size,
prompt length, or activity.

The credible core is:

- one private Rust control daemon;
- one explicit operator CLI;
- one private PostgreSQL database;
- one Velocity proxy as the player entrypoint;
- a small backend topology;
- narrow Velocity integration for commands, sessions, routing, and transfer;
- narrow Paper or Folia integration for backend-owned behavior;
- immutable artifacts and deterministic rendering;
- release-oriented update, rollback, backup, restore, and diagnosis;
- one supported unprivileged Linux system container managed by the already authoritative Incus or
  LXD installation and supervised by systemd.

Do not add an LLM runtime, agent framework, generic workflow engine, second control daemon,
microservice split, Redis, event bus, Kubernetes production path, or distributed coordination merely
because agents develop the repository.

## 2. Authority and evidence

### 2.1 Precedence

Use this default precedence while allowing stronger direct evidence to override weaker material:

1. the user's explicit current request and product policy;
2. the exact active checkout and preserved working tree;
3. executable behavior and built artifacts from the exact relevant revision;
4. current live observations from a named disposable or production environment;
5. tests that cross the affected production boundary;
6. current default branch, recent commit chronology, and actual diffs;
7. current normative specifications and generated owners;
8. current `AGENTS.md`, `README.md`, `docs/work/active.md`, and relevant owner documents within their
   assigned roles;
9. reproducible completion reports and evidence receipts;
10. immutable historical campaigns and prior conversations;
11. old docs, comments, screenshots, and assumptions.

Do not blindly prefer newer prose. Do not preserve an old decision merely because implementing it was
expensive. Do not treat a plan as proof of behavior.

### 2.2 Separate evidence states

Always distinguish:

- local from remote;
- committed from uncommitted;
- source from generated;
- built from packaged;
- packaged from installed;
- installed from running;
- running from ready;
- ready from player-accessible;
- player-accessible from real-player accepted;
- disposable observation from production observation;
- historical observation from current observation.

Use precise evidence language. Relevant states include:

- source inspected;
- implemented;
- formatted;
- statically checked;
- unit tested;
- integration tested;
- process tested;
- PostgreSQL tested;
- generated artifact verified;
- release artifact verified;
- disposable network observed;
- fresh supported-host installed;
- operator observed;
- protocol-client observed;
- real-player observed;
- production observed;
- blocked;
- skipped;
- not run;
- failed;
- deleted;
- deferred.

A lower state never implies a higher state. A skipped, disabled, unset, denied, blocked, or
nonexecuted guard is not a pass. A historical success becomes stale after a relevant source,
artifact, configuration, environment, installation, or deployment change.

### 2.3 Independent observation

Prefer independent oracles at effect boundaries:

- exact file, manifest, digest, and permission inspection;
- process group, executable, systemd, listener, and readiness observation;
- direct PostgreSQL queries and isolated restore;
- protocol clients;
- real players;
- external network vantage points;
- exact remote workflow conclusions.

Tests written from the same implementation are useful but do not automatically prove the external
effect. A database row is not a process action. A rendered configuration is not a loaded plugin. A
spawn is not readiness. A socket is not a player login. A Velocity connection request is not a
completed transfer.

No document may promote evidence through wording.

## 3. Campaign archive, active ledger, and continuity

### 3.1 Immutable campaign history

`docs/campaigns/` contains immutable ChatGPT-to-Codex implementation mandates.

- The campaign explicitly supplied by the user or current task governs that Codex run.
- A later filename is not automatically active.
- Older campaigns are historical context, not cumulative requirements.
- Do not edit a committed campaign because work completed, failed, narrowed, moved, or became
  invalid.
- A later campaign may classify a predecessor as completed, continued, narrowed, terminated,
  superseded, invalidated, blocked, or historical context only.
- When `docs/campaigns/` is absent, create it for the supplied campaign without replacing
  `docs/work/`.
- Campaigns may contain campaign-specific revisions, measurements, blockers, and external boundaries;
  durable policy does not.

### 3.2 Mutable execution ledger

`docs/work/active.md` is the canonical mutable resumable ledger unless a deliberate repository
decision names a successor.

It should contain only current truth:

- current branch and revision;
- governing campaign and predecessor disposition;
- active objective;
- decisions currently in force;
- completed slices and strongest evidence;
- current blockers and untested boundaries;
- external state actually observed;
- one next executable action or a very small ordered set.

Replace stale detail instead of appending an endless diary. Do not duplicate the campaign. Use
campaign files and Git history for history. Do not create `docs/work.md` merely because an old prompt
used that path.

### 3.3 Other document owners

- root `README.md` owns current supported outcomes, onboarding, and honest limits;
- architecture documents own current components, authority, and data/effect flow;
- protocol and contract documents own current semantic and wire behavior;
- operations documents own installation, update, rollback, backup, restore, diagnosis, and recovery;
- release and deployment records own exact artifacts and observed external state;
- generated files are owned by their canonical generators;
- Git history owns code and document changes;
- campaigns own historical implementation intent;
- the active ledger owns current resumption state.

Current code, artifacts, tests, and live evidence outrank historical campaigns about behavior.

## 4. Required read order and context economy

For a new task, read in this order:

1. root `AGENTS.md`;
2. the campaign explicitly supplied by the task;
3. `docs/work/active.md`;
4. root `README.md` and relevant root build/workspace metadata;
5. only the owner documents, source paths, generated owners, tests, workflows, and external evidence
   material to the active objective.

Do not recursively read the entire repository or documentation tree by default. Expand only when a
specific uncertainty requires it. Prefer targeted search, dependency metadata, symbol references,
line ranges, and focused tests over repeated large-file reads.

Do not make a second broad plan when the supplied campaign already resolves the architecture and
order. Begin by reconciling narrow checkout facts, then implement or verify the first dependency.

Use subagents only for independent bounded questions with non-overlapping ownership. Avoid duplicate
scans, builds, downloads, suites, logs, and evidence collection. Keep verbose transient evidence
outside tracked source unless a concise durable record is necessary.

## 5. Work selection and completion

Reconcile active work before opening unrelated work. Choose one of:

- complete the active objective;
- close its missing verification or deployment boundary;
- repair a regression that invalidates accepted evidence;
- narrow it to the smallest dependency-closed completion point;
- terminate or supersede it and delete partial predecessor authority;
- run a narrow evidence or measurement campaign;
- delete or correct a maintained surface that exceeds credible behavior;
- open new product work after prior required journeys are accepted or retired.

Select one observable operator, player, service, or development outcome. Supporting work belongs in
scope only when it is a strict dependency or required cleanup for that outcome.

A coherent completion includes, as relevant:

- implementation;
- maintained-consumer migration;
- generated-owner update;
- one cutover;
- predecessor deletion;
- deterministic verification;
- integration and process proof;
- packaging and release identity;
- installation or deployment proof when claimed;
- rollback and recovery;
- current documentation;
- a resumable ledger;
- evidence-bound handoff.

A narrow local assumption may be corrected without discarding the objective. Record the invalidated
assumption, stronger evidence, conforming adjustment, and material deviation in
`docs/work/active.md`. When evidence invalidates the objective or core design, stop and classify the
state rather than silently weakening acceptance.

## 6. Worktree and Git safety

Before editing, record:

- repository identity and absolute checkout;
- current branch and full commit;
- upstream and remote relation;
- staged and unstaged tracked changes;
- untracked files;
- relevant ignored files;
- nested repositories, worktrees, or submodules;
- preexisting relevant failures.

Preserve unrelated user work. Do not:

- run blind `git reset --hard`;
- run blind `git clean`;
- overwrite uncommitted files to match remote;
- delete unrelated branches or worktrees;
- rewrite shared history;
- force-push;
- move or delete evidence whose ownership is unknown;
- claim a clean worktree without checking it.

Use a dedicated branch or worktree only when it reduces risk and current branch policy permits it.
Do not use branch machinery to hide unresolved state.

Make coherent commits tied to accepted slices. Review the staged diff, generated outputs, tests, and
secret exposure before each durable checkpoint. Push to the intended integration path only after the
relevant gates pass and authentication and branch policy are known.

Never commit secrets, database URLs, token values, private keys, cookies, bearer headers, worlds,
database dumps, unbounded logs, or unnecessary private infrastructure identifiers.

## 7. Compatibility, migration, cutover, and deletion

Backward compatibility is not a default requirement.

This does not authorize destruction of unrelated worktree changes, databases, worlds, backups,
secrets, releases, services, or host state. Before destructive schema, world, release, or deployment
work:

1. identify the exact target;
2. discover maintained readers and writers;
3. create a restorable backup or snapshot;
4. verify it proportionately;
5. define rollback and stop conditions.

When replacing a design:

1. implement one new authority;
2. migrate every maintained reader, writer, generated owner, configuration, route, test, script,
   package, deployment consumer, and owner document;
3. verify the new path;
4. cut over once;
5. delete the predecessor;
6. remove compatibility shims, dual writers, dual readers, shadow state, fallback flags, stale schema,
   registrations, generated branches, tests, docs, CI, scripts, dependencies, and installed artifacts
   owned only by the predecessor;
7. run residual-reference searches across source, generated output, packages, installations, and
   public surfaces.

Do not leave dormant fallback code “just in case” without a current consumer and explicit removal
condition.

## 8. Architecture and ownership

### 8.1 Credible core

Prefer direct typed implementation for current consumers. Do not add a generic planner, controller,
registry, scheduler, event bus, plugin framework, abstraction layer, or service boundary unless a
current demonstrated requirement needs it and the simpler design is inadequate.

The default architecture is:

- Rust daemon: policy, desired state, authorization, durable operations, reconciliation, process
  ownership, readiness aggregation, and private control API;
- CLI: explicit operator client, bounded output, structured output where maintained, nonzero failure,
  and no second business-logic authority;
- PostgreSQL: durable product and operation facts;
- Velocity: public player entrypoint, proxy-owned command/session identity, routing, and actual
  connection requests;
- Paper or Folia plugins: backend-local behavior only;
- systemd: supported service supervision;
- one unprivileged system container: supported production isolation;
- immutable release: deployable identity.

Challenge this direction only with an explicit current user decision or concrete evidence that a
simpler and more valuable architecture exists.

### 8.2 State separation

Keep these states distinct:

- semantic intent;
- persisted desired state;
- durable operation state;
- rendered/generated state;
- installed release state;
- observed process state;
- component readiness;
- player-facing readiness;
- external traffic state.

PostgreSQL must not pretend to own live process truth. A process observation must not silently become
durable desired state. Generated configuration must have one canonical semantic owner and
deterministic renderer.

### 8.3 Effect ownership

Every external effect needs:

- one authority;
- authenticated caller identity;
- authorization;
- stable operation identity;
- bounded deadline;
- cancellation semantics;
- durable state where retry matters;
- idempotency covering the real effect;
- one cleanup owner;
- observable failure;
- recovery and stop conditions.

Do not report success before the effect and its required acceptance oracle succeed.

## 9. Rust control-plane rules

- Prefer small typed modules and explicit dependencies.
- Keep parsing, validation, authorization, planning, persistence, execution, and observation
  separable where this improves independent proof.
- Avoid hidden global mutable state.
- Do not block asynchronous executors with process, filesystem, network, or database work.
- Bound concurrency, queues, request bodies, output, retries, and deadlines for operational reasons,
  not arbitrary aesthetics.
- Propagate cancellation and shutdown.
- Treat timeout as an outcome with explicit durable and retry semantics.
- Return structured, actionable, secret-free errors.
- Do not convert an external effect into success merely because a database transaction committed.
- Use stable operation and resource identifiers.
- Verify exact process ownership before signal, stop, restart, or adoption.
- Stop complete verified owned process groups, not arbitrary PIDs.
- Resist stale PID and PID reuse.
- Distinguish absent, unknown, adopted, fenced, failed, starting, ready, stopping, and stopped states
  when current behavior needs them.
- Bound crash-loop backoff and retain useful transition diagnostics.
- Never shell-interpret service-owned configuration or database environment when direct parsing is
  possible.
- Validate path ancestry, ownership, mode, file type, and identity at privileged boundaries.
- Use no-follow and atomic filesystem operations for privileged publication.
- Preserve the first causal error; cleanup errors must not erase it or delete the only valid state.

Use `unsafe` only when no safe practical alternative exists, isolate it narrowly, state invariants,
and test them.

## 10. PostgreSQL and data rules

PostgreSQL is the durable product store unless a deliberate architecture decision replaces it.

- Migrations are ordered, committed, deterministic, and tested from fresh state and every retained
  upgrade boundary that matters.
- Never edit an already released migration to change history.
- Schema and code cut over together when compatibility is not required.
- Use transactions for multi-step durable state changes.
- Use explicit locking or conflict rules for concurrent mutations.
- Idempotency must cover the external effect, not only duplicate rows.
- Bound database waits and pool acquisition.
- Do not hold transactions open across slow external effects without a justified protocol.
- Use database constraints for durable invariants and application checks for contextual invariants;
  do not create duplicate conflicting authorities.
- Separate desired state, operation state, and observed runtime state.
- Destructive migration requires exact target identity, verified backup, rollback classification, and
  tested restore.
- When a migration makes old binaries incompatible, binary-only rollback is forbidden.
- Retain failed data until a replacement or restored target is accepted.

Backups must be transaction-consistent, private, checksummed, metadata-bound, and independently
inspectable. Restore into a fresh isolated target before claiming recoverability. A successful
`pg_dump`, copied file, checksum alone, or `pg_restore` exit alone is not service recovery.

Never log or retain a full database URL or secret.

## 11. Protocol, authentication, and network rules

- Use one canonical semantic contract and one generated wire owner where generation is maintained.
- Version protocol behavior explicitly.
- Reject malformed, unauthorized, stale, duplicate, expired, unsupported, and oversized requests
  truthfully.
- Define deadlines, cancellation, retry, and idempotency at the real effect boundary.
- Keep authentication and authorization separate.
- The daemon is final for policy, desired state, authorization, and durable operations.
- Velocity owns authenticated proxy player identity, proxy command registration, proxy sessions,
  routing, and actual connection requests.
- Backend plugins must not invent proxy sessions or assert transfer completion.
- Credentials are narrowly scoped to one principal and effect.
- Store credentials outside release bytes and source.
- Use least-privileged file ownership and modes.
- Re-read or rotate credentials according to the current owner contract without exposing values.
- Redact secrets from errors, logs, tests, evidence, and handoffs.
- Do not accept caller-supplied identity that the platform can derive authoritatively.
- Keep daemon, PostgreSQL, backend listeners, credentials, and management interfaces private.
- Expose only the intended Velocity player listener unless a later explicit requirement justifies
  another public boundary.
- Verify exposure from the container, host, and external vantage points relevant to the claim.

A connection request is not a completed transfer. A completion suggestion based on stale session or
backend state must be suppressed or revalidated according to the current contract.

## 12. Velocity, Paper, and Folia rules

Never block Velocity event loops or Paper/Folia scheduler-owned threads with network, database,
filesystem, process, or long-running work.

For platform events:

1. capture immutable event and identity data before asynchronous work;
2. perform slow work on bounded owned execution;
3. propagate deadline, cancellation, and shutdown;
4. revalidate player, connection, server, entity, world, region, and generation before applying a
   late result;
5. return to the platform's correct scheduler for platform mutation;
6. suppress stale completion, menu, transfer, or feedback results;
7. remove registrations and stop owned work on disable;
8. make lifecycle replacement idempotent and leak-free.

Velocity rules:

- register proxy commands and completions through Velocity's supported command manager;
- use real `Player` identity for player effects;
- use the platform connection-request result as the transfer observation;
- bound in-flight status, completion, transfer, and feedback work;
- preserve permits until the underlying operation settles when timeout feedback does not cancel it;
- do not mutate platform state from arbitrary completion threads.

Paper/Folia rules:

- keep backend behavior backend-local;
- do not perform proxy policy or session authority;
- use the correct global, region, entity, or async scheduler;
- revalidate late results;
- keep Paper and Folia support claims separate;
- Paper proof does not imply Folia proof;
- a mock scheduler does not imply live platform safety.

Generated JVM protocol or model code must be regenerated from its canonical owner. Do not hand-edit a
generated file to create a second authority.

## 13. Process, readiness, and reconciliation rules

- Process identity must resist stale PID and PID reuse.
- Persist enough identity to classify a process, but re-observe live truth before acting.
- Treat desired state, durable operation state, process existence, component readiness, and public
  readiness separately.
- Spawn is not readiness.
- Port bind is not necessarily Minecraft readiness.
- Readiness must be bounded and component-appropriate.
- A backend is joinable only when the current policy's process, protocol, registration, and heartbeat
  evidence agree.
- Daemon restart must classify existing processes as adopted, fenced, failed, absent, or unknown
  truthfully.
- Stop and restart only verified owned cgroups/process groups.
- A surviving unverified process blocks destructive reuse of its resources.
- Reconciliation retries only classified transient conditions and surfaces persistent blockers.
- No-op reconciliation must preserve process identities and avoid unnecessary writes.
- Service restart and container restart require fresh post-restart observation.
- Keep transition logs bounded and secret-free.

systemd is the supported production supervisor. Unit behavior, dependencies, restart policy, cgroup
ownership, readiness, writable paths, and logging must be explicit and verified in the packaged and
installed unit.

## 14. Artifacts, release, update, rollback, backup, and restore

### 14.1 Acquisition and build

- Pin immutable upstream identities and verify digests.
- A mutable channel may be a resolution input but never installed identity.
- Bound redirects, timeouts, size, and extraction.
- Download into private temporary files.
- Verify before extraction or atomic cache publication.
- Treat cached filenames as hints, not integrity proof.
- Do not silently repair corruption by trusting another mutable source.
- Release builds use an exact clean source object and fresh private outputs.
- Do not consume ambient `target/`, Gradle outputs, ignored files, or caller build caches as release
  authority.
- Source export must be independently importable and tied to the exact declared commit.
- Fail closed when source objects, manifests, pins, or dependencies are unavailable.

### 14.2 Manifest and release identity

A release manifest must be checked against an independently authored inventory. Verify exact set
equality, paths, regular-file type, ownership expectations, size, digest, provenance, and required
embedded identity. Reject missing, extra, duplicate, traversing, symlinked, special, mutable, or
unowned entries.

Bind:

- source commit;
- binaries;
- JVM jars;
- schema/configuration/protocol identity;
- operational tools;
- canonical systemd owners;
- artifact checksums;
- compatibility and rollback facts where needed.

Keep source bytes, build tools, secrets, worlds, logs, backups, and runtime state out of production
release dependencies.

Published releases, when explicitly authorized, attach the exact accepted bundle rather than
rebuilding it.

### 14.3 Installation and update

A supported installation or update must:

- verify exact target identity before mutation;
- separate immutable release bytes from configuration, secrets, worlds, logs, backups, and runtime
  files;
- stage privately on the target filesystem;
- fsync and activate atomically where durability matters;
- never overwrite a differing immutable versioned target;
- retain a verified previous release;
- serialize mutations;
- create and verify required backup before destructive effects;
- define interruption and partial-success states;
- leave one current authority;
- verify installed identity after activation;
- verify running identity and readiness separately.

Do not build Rust or Gradle source, resolve `latest`, or silently replace jars at service startup.

An identical update should be a true verified no-op. It must not back up, migrate, restart, or
republish merely to confirm identity.

### 14.4 Fencing and recovery

When an update can stop or replace the service:

- create durable private operation state before ambiguity;
- fence automatic restart before publication;
- use any one-use start permission narrowly and atomically;
- preserve old and new exact release identity;
- classify interruption;
- recover through the exact packaged tool;
- remove fence only after independent acceptance;
- never delete the only valid release or data copy.

When data migration changed, do not roll back only binaries. Use the matching verified backup or
snapshot and matching release.

### 14.5 Backup and restore

Backup creation and restore verification are separate. Validate checksums, metadata, versions,
schema, migration identity, and application behavior. Restore into a fresh target. Preserve the
source and failed target until the restored service is accepted. Define atomic cutover and rollback
for any production restore.

## 15. Deployment and host safety

Live-discover the target. Do not assume historical host, container, address, storage, listener, or
service state.

- Use the one container manager already authoritative on the host.
- Do not operate Incus and LXD as competing authorities.
- Use an unprivileged system container with explicit resource limits based on current capacity.
- Use systemd for the supported path.
- Use least-privileged service and PostgreSQL roles.
- Deploy exact immutable bytes with manifest and checksums.
- Keep source checkout and build toolchains out of runtime dependencies.
- Keep releases, configuration, secrets, worlds, logs, backups, and runtime state separated.
- Verify service dependencies, restart behavior, cgroup/process ownership, readiness, writable paths,
  file permissions, and bounded logs.
- Keep daemon, database, backends, and management interfaces private.
- Expose only intended Velocity player traffic.
- Verify listeners from relevant container, host, LAN, and external vantage points.
- Preserve application rollback and traffic rollback separately.
- Verify unrelated services after network, firewall, proxy, storage, or container-manager changes.
- Do not use privileged containers, host networking, broad mounts, unrestricted manager sockets,
  Docker sockets, nested containers, or direct host mutation for convenience.
- Never fabricate Minecraft EULA acceptance or any third-party consent.
- Never treat an old deployment record as current without live observation.

Before destructive external work, establish authorization, exact target, backup, verified restore or
snapshot, rollback, stop conditions, and protected unrelated state.

## 16. Testing and verification

### 16.1 Test the real changed boundary

Use the smallest sufficient progression:

1. formatting and static checks;
2. focused unit tests;
3. focused integration tests;
4. real PostgreSQL tests;
5. process/systemd tests;
6. generated artifact and package inspection;
7. disposable network/host observation;
8. operator observation;
9. protocol-client observation;
10. real-player observation;
11. production observation.

Use only tiers relevant to the objective, but never claim an unrun higher tier.

Test positive behavior and objective-critical negative behavior:

- malformed and unauthorized requests;
- stale, duplicate, expired, and unsupported inputs;
- timeout and cancellation;
- queue or pool saturation where changed behavior can trigger it;
- database outage, conflict, and incompatible schema;
- stale PID, PID reuse, surviving child, and restart;
- digest mismatch, corrupt artifact, interrupted publication, and disk/resource failure where
  relevant;
- plugin disable, scheduler shutdown, disconnect, reconnect, and late result;
- interrupted update, no-op, rollback, and restore.

Prefer bounded deterministic fault tests and disposable environments over a permanent chaos
framework.

### 16.2 Preserve original failure

No retry, cleanup, diagnostic, or evidence step may erase the original exit status. `continue-on-error`
does not make a required gate pass. Cleanup still runs, and cleanup failure is reported separately.

A success marker must be produced by the real owner after all required checks. Evidence parsers fail
closed on missing, malformed, conflicting, or incomplete results.

### 16.3 Fresh final proof

- Run focused checks during iteration.
- Run fresh final checks after the final relevant change.
- Rebuild generated and release artifacts after any input change.
- Bind final evidence to exact source, commit, release, installation, or deployment.
- Relevant changes stale earlier live evidence.
- Inspect packaged and installed bytes when source tests cannot prove them.
- Keep logs bounded and redacted.
- Record commands and exit status.
- Separate reused historical evidence from fresh evidence.

Do not run an expensive whole-product or live gate repeatedly when it cannot add confidence, but do
not omit the real affected boundary to save time.

## 17. Performance

Measure before optimizing.

Measure the real player or operator boundary with revision, environment, workload, warm-up,
repetitions, and durations. Use repeated samples when variance matters. Profile the observed dominant
cost before changing architecture.

Prefer:

- deletion;
- fewer processes;
- fewer round trips;
- bounded work;
- explicit queries;
- smaller artifacts;
- simpler verification;
- fewer conversions and copies.

Do not add caching, custom executors, concurrency, queues, Redis, sharding, microservices, or
distributed coordination without a measured need. Do not trade authentication, durability, truthful
failure, recovery, scheduler safety, or determinism for cosmetic speed. Remove rejected speculative
optimization machinery.

## 18. Documentation

Update source, tests, generated owners, and concise current owner documents in the same coherent
slice.

Documentation must state:

- current supported outcome;
- authority;
- inputs and outputs;
- failure behavior;
- recovery;
- evidence tier;
- unsupported boundaries.

Do not:

- copy a campaign into owner docs;
- copy the active ledger into README;
- make current docs claim a future deployment;
- present historical host observations as current;
- duplicate one contract in several prose owners;
- retain removed commands, routes, schema, configuration, scripts, or fallback behavior;
- use documentation volume as a substitute for implementation or proof.

Use plain established terms. Avoid unnecessary project jargon, evidence codes, contract numbers,
arbitrary line limits, file-count rules, or planning abstractions.

## 19. Security and secrets

- Use least privilege.
- Authenticate every privileged or player-sensitive effect.
- Authorize at the final policy owner.
- Scope credentials to one principal and purpose.
- Keep credentials outside source and release bytes.
- Validate ownership, modes, ancestry, file type, and identity before privileged reads or execution.
- Use no-follow traversal and atomic publication at hostile path boundaries.
- Bound request bodies, output, redirects, downloads, archive members, file counts, depth, and
  resource use according to operational evidence.
- Reject traversal, symlinks, special files, races, and mutable identities where the contract requires
  regular immutable files.
- Redact secrets and private data from diagnostics and evidence.
- Scan release and retained evidence for generated canaries and credential values.
- Do not weaken checks because the current environment is inconvenient.
- Do not expose the control daemon or database publicly.
- Never place host/container manager sockets inside the workload.
- Treat legal consent as an external human-owned prerequisite.

## 20. Agent and API-cost discipline

Use ChatGPT for upstream comparison and design; use Codex for active-checkout facts, implementation,
execution, and proof.

Minimize constrained-agent cost by:

- reading the prescribed small initial set;
- searching for exact symbols and owners;
- avoiding repeated broad scans;
- avoiding overlapping subagent assignments;
- running focused checks before full suites;
- reusing verified immutable downloads without treating cache as authority;
- collecting evidence once at the strongest useful boundary;
- keeping the active ledger concise;
- deleting obsolete paths that future agents would otherwise rediscover.

Do not save tokens by leaving broad design ambiguity, fake acceptance, missing rollback, or duplicate
authority. Durable clarity and architectural contraction reduce future cost more than a short but
underspecified handoff.

## 21. Required handoff

Every substantive task handoff must include:

- objective and disposition;
- starting and final branch, full commit, upstream relation, and worktree state;
- governing campaign and active-ledger path;
- behavior and authorities changed;
- files, generated owners, schema, protocol, configuration, packaging, and deployment consumers
  changed;
- predecessor paths and compatibility scaffolding deleted;
- exact commands and exit status;
- targeted and final verification;
- generated and release artifact identity;
- backups, restore checks, rollback state, and destructive actions;
- installation and deployment changes;
- process, listener, database, operator, protocol-client, real-player, and production observations
  kept separate;
- skipped, blocked, not-run, failed, deferred, and reused evidence;
- deviations from campaign assumptions and supporting proof;
- sensitive actions described with secrets redacted;
- remaining risks and unsupported boundaries;
- one next executable action, or a precise statement that the objective is closed.

Do not collapse several evidence states into an unqualified word such as `complete`, `working`, or
`deployed`.

Leave the repository in one of two states:

1. clean, buildable, verified, committed, and integrated according to current authorization; or
2. stable and resumable, with exact blocker, preserved data/evidence, no hidden partial authority, and
   one executable next action.
