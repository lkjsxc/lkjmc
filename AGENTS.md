# AGENTS.md

## 1. Purpose and scope

This file defines durable repository-wide policy for agents working in lkjmc. A more specific
`AGENTS.md` may add rules for its subtree, but it must not weaken repository-wide safety, evidence,
data, release, or deployment requirements.

lkjmc should remain a small, truthful, release-oriented Minecraft control plane that future AI coding
agents, including weaker models, can understand and maintain. Optimize for real operator and player
outcomes, correctness, recoverability, architectural contraction, proof at the affected boundary, and
useful evidence per unit of agent work. Do not optimize for apparent sophistication, feature count,
diff size, prompt length, activity, or novelty.

The credible core is:

- one private Rust control daemon;
- one explicit operator CLI;
- one private PostgreSQL database;
- one Velocity proxy as the public player entrypoint;
- a small backend topology;
- narrow Velocity integration for commands, sessions, routing, and transfer;
- narrow Paper or Folia integration for backend-owned behavior;
- immutable artifacts and deterministic rendering;
- release-oriented update, rollback, backup, restore, and diagnosis;
- one supported unprivileged Linux system container managed by the already authoritative Incus or
  LXD installation and supervised by systemd.

Challenge this direction only when the user makes a current product decision or concrete evidence
shows that a simpler and more valuable architecture exists.

Do not add an LLM runtime, agent framework, generic workflow engine, second control daemon,
microservice split, Redis, event bus, Kubernetes production path, service mesh, distributed
coordination, or speculative abstraction merely because agents develop the repository.

## 2. Authority and evidence

### 2.1 Default precedence

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
expensive. Do not treat a plan, status page, generated receipt, or test name as proof of behavior it
did not observe.

### 2.2 Separate evidence states

Always distinguish:

- local from remote;
- committed from uncommitted;
- source from generated;
- built from packaged;
- packaged from retained or published;
- retained from independently retrieved;
- retrieved from verified;
- verified from installed;
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

A lower state never implies a higher state. A skipped, disabled, unset, denied, blocked, cancelled, or
nonexecuted guard is not a pass. A historical success becomes stale after a relevant source,
artifact, configuration, environment, installation, or deployment change.

### 2.3 Independent observation

Prefer independent oracles at effect boundaries:

- exact file, manifest, digest, archive-member, and permission inspection;
- process group, executable, systemd, listener, and readiness observation;
- direct PostgreSQL queries and isolated restore;
- protocol clients;
- real players;
- external network vantage points;
- exact remote workflow conclusions;
- independent artifact retrieval after storage or publication.

Tests written from the same implementation are useful but do not automatically prove the external
effect. A database row is not a process action. A rendered configuration is not a loaded plugin. A
spawn is not readiness. A socket is not a player login. A Velocity connection request is not a
completed transfer. A retained manifest is not retained release bytes. An uploaded artifact is not an
installed release.

No document may promote evidence through wording.

### 2.4 Statement discipline

Treat every material claim as one of:

- verified current fact;
- current user policy;
- inherited durable rule still justified;
- historical context;
- selected design mandate;
- narrow empirical question;
- implementation latitude;
- deferred or out of scope.

Do not convert a suspicion into a requirement, a policy into a suggestion, an implementation accident
into architecture, or historical evidence into current external state.

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
line ranges, recent diffs, and focused tests over repeated large-file reads.

Do not make a second broad plan when the supplied campaign already resolves the architecture and
order. Begin by reconciling narrow checkout facts, then implement or verify the first dependency.

Use subagents only for independent bounded questions with non-overlapping ownership. Avoid duplicate
scans, builds, downloads, suites, logs, and evidence collection. Keep verbose transient evidence
outside tracked source unless a concise durable record is necessary.

## 5. Work selection and completion

Reconcile active work before opening unrelated work. Choose one of:

- complete the active objective;
- close its missing verification, artifact, installation, or deployment boundary;
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
- artifact retention and independent retrieval when later consumption is claimed;
- installation or deployment proof when claimed;
- rollback and recovery;
- current documentation;
- a resumable ledger;
- evidence-bound handoff.

A narrow local assumption may be corrected without discarding the objective. Record the invalidated
assumption, stronger evidence, conforming adjustment, and material deviation in
`docs/work/active.md`. When evidence invalidates the objective or core design, stop and classify the
state rather than silently weakening acceptance.

Prefer a small campaign that fully closes the primary bottleneck over a broad campaign that leaves
parallel systems, ambiguous authority, or unverifiable claims.

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

### 8.2 State separation

Keep these states distinct:

- semantic intent;
- persisted desired state;
- durable operation state;
- rendered/generated state;
- built artifact state;
- retained transport state;
- installed release state;
- observed process state;
- component readiness;
- player-facing readiness;
- external traffic state.

PostgreSQL must not pretend to own live process truth. A process observation must not silently become
durable desired state. Generated configuration must have one canonical semantic owner and
deterministic renderer. Artifact storage must not silently become installed identity.

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

### 8.4 Authority inventory

For every material change identify:

- semantic owner;
- persisted owner;
- generated owner;
- observed-runtime owner;
- external owner;
- readers;
- writers;
- maintained consumers;
- failure owner;
- recovery owner;
- evidence owner.

Do not create two authorities for the same rule because migration is inconvenient.

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
- Do not use `unwrap`, `expect`, panic, or silent defaulting for recoverable production input or effect
  failures.

Use `unsafe` only when no safe practical alternative exists, isolate it narrowly, state invariants,
and test them.

## 10. Process, readiness, and lifecycle rules

- Desired state, durable operation state, process identity, observed readiness, and public readiness
  are separate.
- Spawn is not readiness. Port bind is not necessarily Minecraft readiness.
- Readiness must be bounded, component-appropriate, and independently observable.
- Process identity must resist stale PID files and PID reuse.
- Stop and restart only verified owned process groups or systemd units.
- After daemon restart, classify existing processes as adopted, fenced, failed, absent, or unknown
  truthfully.
- Do not adopt a process whose executable, cgroup, start time, ownership, or expected identity cannot
  be verified.
- A timeout or cancellation must leave explicit durable state and a safe retry or stop condition.
- Crash loops require bounded backoff and useful diagnostics.
- Child survival after parent failure must be observed and handled, not assumed away.
- Service restart and container restart are separate evidence boundaries.
- systemd state, application readiness, and player-facing readiness must not be collapsed.

## 11. PostgreSQL and data rules

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
- Use parameterized or prepared queries; do not interpolate untrusted input.
- Keep schema, query, and transaction ownership explicit.
- Do not store opaque serialized domain state when normalized durable facts are required for
  constraints, migration, recovery, or diagnosis.

Backups must be transaction-consistent, private, checksummed, metadata-bound, and independently
inspectable. Restore into a fresh isolated target before claiming recoverability. A successful
`pg_dump`, copied file, checksum alone, or `pg_restore` exit alone is not service recovery.

Never log or retain a full database URL or secret.

## 12. Protocol, authentication, and network rules

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
- Verify exposure from the container, host, and external vantage points when deployment claims it.
- A connection request is not a completed transfer; observe the same player's destination or another
  appropriate independent oracle.
- Bound command completion, menu, status, and transfer requests; stale results must not mutate current
  sessions.

## 13. JVM, Velocity, Paper, and Folia rules

- Never block Velocity event loops or Paper/Folia scheduler-owned threads with network, database,
  filesystem, process, or long-running work.
- Capture immutable event data before asynchronous work.
- Bound concurrency, queues, deadlines, retries, cancellation, and shutdown.
- Return to the correct platform scheduler before platform mutation.
- Revalidate player, connection, entity, world, chunk, and generation identity before applying late
  results.
- Do not retain platform objects across asynchronous boundaries unless the platform contract permits
  it and revalidation remains possible.
- Velocity owns proxy commands, proxy sessions, routing, and actual connection requests.
- Paper/Folia plugins own backend-local behavior only.
- Shared JVM code must not smuggle platform-thread assumptions across modules.
- Keep Paper and Folia claims separate. Paper proof is not Folia proof.
- Plugin startup logs may report exact build identity but may not claim readiness before registration,
  dependency, and effect checks pass.
- Plugin disable or scheduler shutdown must cancel owned work and reject late mutation.

## 14. Configuration, generation, and static data

- Give every generated file one canonical generator and one semantic owner.
- Generation must be deterministic for identical inputs.
- Check generated output for drift in CI when it is maintained.
- Do not hand-edit generated output unless the owner contract explicitly requires it.
- Validate configuration with a schema or typed parser before effect execution.
- Unknown, duplicate, conflicting, traversing, or unsafe configuration fails explicitly.
- Keep examples current, secret-free, and executable where practical.
- Separate desired configuration from rendered files and observed runtime adoption.
- Remove obsolete schema branches, generated compatibility code, fixtures, docs, and tests during
  cutover.
- Do not allow mutable remote catalogues to become runtime authority for installed artifacts.

## 15. Artifacts, release identity, and transport

### 15.1 Build and release identity

- Build from a clean exact source commit.
- Do not use caller ambient build outputs, ignored files, caches, or unpublished parent objects as
  release authority.
- Pin toolchains, dependency locks, base images, wrapper distributions, and upstream artifact
  identities according to their current owners.
- Verify acquired bytes before use.
- A mutable label may resolve an artifact but is not installed identity.
- Generate one independently verifiable manifest that binds exact source, artifacts, checksums,
  configuration or schema identity, and compatibility information required by current consumers.
- Verify built binaries and jars expose the exact intended identity.
- Build reproducibility must compare independent fresh outputs in the same declared environment.
- Host-native and pinned-container outputs are different environments unless proven otherwise.
- Do not normalize unexplained binary differences into success.
- Secret-scan source context, release bytes, image layers, and retained evidence before publication.
- A successful scan is a prerequisite, not a cleanup note.

### 15.2 Artifact retention is a separate boundary

A manifest, checksum file, provenance record, log, test receipt, or successful workflow conclusion is
not the release bytes.

When a later operator, installer, updater, or deployment is expected to consume exact accepted bytes:

- retain or publish the exact accepted bytes rather than silently rebuilding them later;
- use a permission-preserving inner package when the outer transport can rewrite modes or metadata;
- give the package an immutable content digest and exact source identity;
- keep transport metadata distinct from installed-content identity;
- independently retrieve the retained artifact after storage or publication;
- reverify archive, manifest, file set, modes, embedded identity, and secret boundary after retrieval;
- record retention or expiry honestly;
- do not call metadata-only evidence an available release;
- do not call an uploaded artifact installed or deployed;
- attach the exact accepted bundle to any later published release rather than rebuilding it.

The outer storage service, artifact ID, archive digest, release-manifest digest, installed release
root, and running release are related but distinct identities.

### 15.3 Archive and extraction safety

When an archive is maintained:

- use one canonical packer and verifier;
- derive contents from the independent release inventory;
- reject absolute paths, traversal, duplicate members, links, special files, unexpected extensions,
  unstable input identity, and unbounded content;
- make deterministic metadata explicit;
- preserve required executable and data modes;
- write through private temporary state and atomically publish;
- never overwrite an ambiguous target;
- validate all members before extraction;
- extract through no-follow descriptor-relative operations into a new private target;
- fsync before acceptance where durability is claimed;
- clean only operation-owned partial state;
- run the normal manifest and embedded-identity verifier after extraction.

Do not rely on a convenience `tar -xf`, ZIP extraction, or artifact action to provide these
properties implicitly.

### 15.4 Publication and signing

- Publication is a distinct external action with explicit authorization.
- Published artifacts must be the exact verified artifacts, not a later rebuild.
- Checksums bind bytes, not publisher identity.
- Signing requires a separately trusted key, explicit key custody, verified signatures, and rotation
  and revocation semantics.
- Absence of signing is an explicit unsupported or skipped boundary, not an implied signature.
- Do not publish secrets, private evidence, worlds, database dumps, or host-specific configuration.
- Artifact expiration changes availability, not historical byte identity.

## 16. Installation, update, rollback, backup, and restore

- Production services do not compile Rust or Gradle source, resolve `latest`, or replace jars at
  startup.
- Installation verifies the exact bundle, manifest, digest, file set, modes, and target identity before
  mutation.
- Separate immutable release bytes from configuration, secrets, worlds, logs, backups, and runtime
  state.
- Install into a versioned nonconflicting target and activate atomically.
- Keep enough exact previous release state for safe rollback when rollback is supported.
- New update, exact no-op, interrupted update, restart adoption, rollback, and failed rollback are
  separate outcomes.
- Exact no-op must not back up, migrate, stop, restart, switch pointers, or rewrite artifacts.
- Before a changed update, create the required private verified backup and record rollback state.
- Use one global deployment lock and durable operation/fence state where current design requires it.
- A crash or reboot during update must not bypass a durable fence.
- Recovery must use exact packaged authority, not an ad hoc checkout tool.
- Restore data and matching release identity together when migration compatibility requires it.
- Binary-only rollback is forbidden after an incompatible schema transition.
- Verify restored data in an isolated target before claiming recoverability.
- Preserve the previous valid state when cleanup of a newly accepted state fails.
- Do not fabricate host snapshot observation from inside a container.
- Keep failure receipts actionable, bounded, and secret-free.

## 17. Supported-host deployment

- Live-discover the authorized target; do not reuse historical hostnames, addresses, container names,
  storage pools, or routes without verification.
- Use the one container manager already authoritative on the host, Incus or LXD. Do not operate two
  competing managers.
- Use an unprivileged system container with explicit resources justified by current capacity and
  workload.
- Do not use privileged containers, host networking, broad host mounts, unrestricted manager sockets,
  or direct host mutation merely for convenience.
- Keep source checkout and build toolchains out of production runtime dependencies.
- Use least-privileged service and PostgreSQL roles.
- Use systemd for the supported service path, including dependencies, restart behavior, writable-path
  restrictions, readiness, and bounded logging.
- Keep daemon, PostgreSQL, backends, credentials, and management listeners private.
- Expose only the intended Velocity player listener unless explicit current policy requires more.
- Verify listeners from container, host, and external vantage points.
- Preserve unrelated host services, firewall rules, DNS, proxying, storage, containers, and workloads.
- Establish exact target identity, capacity, backup, rollback, credentials, consent, and traffic
  isolation before mutation.
- Never fabricate Minecraft EULA acceptance or other third-party consent.
- Separate disposable proof from production proof.
- After network or firewall changes, verify unrelated services and retain rollback.

## 18. Testing and verification

### 18.1 Test order

Use the cheapest relevant proof while iterating, then run fresh final proof after the final relevant
change.

Typical progression:

1. parser, type, and unit tests;
2. generated-output and static contract checks;
3. focused integration tests;
4. PostgreSQL tests;
5. process and filesystem fault tests;
6. release construction and artifact inspection;
7. independent artifact retrieval and verification;
8. disposable network or supported-host tests;
9. operator observation;
10. protocol-client observation;
11. real-player observation;
12. production observation.

Do not run expensive broad or live gates when they cannot add new confidence. Do not skip a required
boundary because lower tests are green.

### 18.2 Failure coverage

At changed effect boundaries test relevant:

- malformed and unauthorized input;
- stale, duplicate, expired, and unsupported requests;
- timeout, cancellation, disconnect, reconnect, and shutdown;
- queue saturation and bounded backpressure;
- partial writes and interrupted publication;
- digest mismatch, corruption, truncation, and disk exhaustion;
- database outage, lock conflict, pool exhaustion, and incompatible schema;
- stale PID, PID reuse, process crash, child survival, and restart;
- archive traversal, duplicate entries, links, special files, wrong modes, and extraction conflict;
- missing, expired, or altered retained artifacts;
- updater interruption, no-op, rollback, and recovery;
- plugin disable and scheduler shutdown;
- container or host restart when the objective claims it.

Prefer bounded deterministic fault injection and disposable environments over elaborate chaos
infrastructure.

### 18.3 Test honesty

- Do not hide required tests behind ignored, unset, denied, or unavailable guards and report pass.
- A guarded lane that did not execute is `SKIPPED` or `BLOCKED`.
- Record preexisting failures separately from introduced failures.
- Tests must not merely restate implementation internals when an independent oracle is practical.
- Inspect built binaries, jars, archives, manifests, checksums, modes, listeners, ownership, and
  permissions when source tests cannot prove them.
- Bind final evidence to exact final source, artifact, installation, or deployment identity.
- Keep logs bounded and redacted.
- Remove temporary resources and verify cleanup.

## 19. Performance and resource use

Measure before optimizing.

- Measure the real operator or player boundary, not a convenient internal proxy.
- Record revision, environment, workload, warm-up, repetitions, and durations.
- Use repeated samples or distributions rather than one warm run.
- Profile the observed dominant cost.
- Prefer deletion, fewer processes, fewer round trips, explicit queries, smaller artifacts, and
  bounded work before caches or concurrency.
- Do not add arbitrary timeouts, pool sizes, thread counts, queue depths, file limits, or resource
  budgets without an operational reason and evidence.
- Do not trade authentication, durability, truthful failure, scheduler safety, recovery, artifact
  integrity, or determinism for speed.
- Rerun the same benchmark after each retained optimization.
- Record rejected optimizations when lack of evidence matters to future work.

Performance work belongs in a campaign only when it is the observed bottleneck or a strict acceptance
requirement.

## 20. Documentation rules

- Keep root `AGENTS.md` durable. Do not put current commits, versions, hosts, addresses, container
  names, artifact IDs, measurements, objective, blockers, or deployment status here.
- Keep `docs/work/active.md` current and concise.
- Keep committed campaigns immutable.
- Update source, tests, generated owners, and concise owner docs in the same coherent slice.
- Do not duplicate large implementation plans across docs.
- Do not claim future release, artifact, installation, deployment, client, player, or production state
  as present.
- Mark unsupported and unobserved boundaries honestly.
- Delete obsolete docs, examples, diagrams, runbooks, and generated references during cutover.
- Preserve exact commands only when they are maintained operator interfaces or reproducible evidence,
  not transient scratch history.
- Keep private infrastructure identities and secrets out of tracked docs.
- Use established terminology and plain precise English; avoid unnecessary internal jargon or
  evidence-code proliferation.

## 21. Agent economy and implementation discipline

- Use upstream design to avoid making Codex repeat broad architectural exploration.
- Read the smallest sufficient source set and expand only for concrete uncertainty.
- Prefer targeted searches and line ranges over repeated full-file reads.
- Do not duplicate builds, test suites, downloads, scans, or artifact retrieval without a new
  evidence purpose.
- Reuse verified immutable artifacts instead of rebuilding when later consumers require the same
  bytes.
- Keep subagent tasks independent, bounded, and nonoverlapping.
- Do not ask a subagent to rediscover decisions already made by the governing campaign.
- Keep transient logs outside tracked source unless concise durable evidence is needed.
- Do not create planning frameworks, status databases, generated dashboards, or registries to manage
  one campaign.
- Make the first implementation slice produce value or reduce objective-critical uncertainty; do not
  spend the first turn rewriting the campaign into another plan.
- Prefer direct deletion and simplification over adding abstractions to contain obsolete paths.

## 22. External actions and authorization

### 22.1 Normally permitted after inspection

When the governing campaign requires them and prerequisites are satisfied:

- source edits;
- coherent commits;
- push to the intended branch or review path;
- pinned upstream downloads;
- deterministic release assembly;
- bounded workflow artifact upload/download;
- disposable local fixtures;
- deletion of obsolete lkjmc-owned paths.

### 22.2 Require objective-specific prerequisites

The governing campaign must explicitly require and guard:

- tags and GitHub Releases;
- signing;
- installation;
- Incus/LXD or host mutation;
- service stop/restart;
- database migration, backup, or restore;
- listener, firewall, DNS, proxy, or traffic changes;
- production cutover;
- destructive data or world changes.

Before those actions establish exact target, authorization, identity, credentials, backup, rollback,
capacity, isolation, and stop conditions.

### 22.3 Forbidden by default

- force-push;
- destructive shared-history rewrite;
- blind reset or clean;
- unrelated branch, worktree, host, service, network, data, or artifact deletion;
- mutation outside lkjmc scope;
- secret disclosure;
- fabricated legal consent;
- silent production cutover;
- rebuilding after final verification and publishing the different bytes;
- claiming an unavailable external tier passed.

Routine technical choices already resolved by the campaign do not require user reconfirmation. Stop
or ask only for a genuinely unavailable secret, legal consent, physical action, ambiguous target, or
destructive action outside established authority.

## 23. Completion and handoff

Leave either:

- a clean, buildable, accepted state; or
- a precise, safe, resumable blocked state with no hidden partial authority.

The final handoff must include, as relevant:

- objective and final disposition;
- starting and final branch, commit, remote relation, and worktree state;
- governing campaign and installed policy paths;
- changed behavior, files, generated owners, schema, protocol, configuration, and workflow;
- predecessor paths deleted;
- migration, backup, restore, and rollback state;
- exact commands and exit status;
- targeted and final verification;
- release root, archive, manifest, artifact-service, installation, and deployment identities kept
  separate;
- artifact retention, retrieval, and expiry when applicable;
- process, listener, database, operator, protocol-client, real-player, and production observations
  kept separate;
- skipped, blocked, not-run, failed, and reused evidence;
- deviations from assumptions and supporting proof;
- sensitive actions, appropriately redacted;
- remaining risks and unsupported boundaries;
- one next executable action, or a precise statement that the objective is closed.

Do not collapse multiple evidence states into an unqualified word such as `complete`, `released`, or
`deployed`.
