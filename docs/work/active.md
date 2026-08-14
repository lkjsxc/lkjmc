# Active work

## Current objective

Exact release `b6d22115f1726aeb570e91900cabcc008ca55689` still serves the five-route local Paper/Folia menu, migration 53, scoped heartbeat, and truthful backend readiness. The checkout installer is withdrawn in source commit `d360c1f33e19077e5f09925dc97ff05617109e57` and a release-packaged, externally anchored, update-only coordinator is implemented. The first exact `31de8b9` disposable-LXC preflight failed before mutation because Ubuntu exposes `/usr/bin/psql` as a root-owned symlink; `7a531a109b9ee0212961493513b3145531e2948e` now resolves only root-owned command symlinks to root-owned executable targets under non-writable ancestry. The coordinator also requires a fresh verified PostgreSQL backup, global lock, durable systemd restart fence, migration-aware rollback/recovery, atomic plugin/unit/current publication, real proxy ping, and verified no-op. The initial independent review found root shell execution, fake-backup acceptance, missing locking/fencing, mutable release paths, unsafe plugin/state paths, stop recovery, late publisher cleanup, EULA disagreement, and duplicate topology acceptance; those findings were repaired. Deterministic checks pass, but the updater is not accepted until exact `b6d2211` adoption, no-op, restart, and backup/restore run in a disposable unprivileged LXC and then production. No authorized online-mode player has exercised `/menu`, `/docs`, `/lkjmc` completion/status, or transfer.

## Repository state

- Recovery base: `339adb7bb60b6cce5229f98866f869363e90b78b`.
- Product and deployed release commit: `b6d22115f1726aeb570e91900cabcc008ca55689`; prior heartbeat release: `358b27d4bc907b7d606a64447fae064e44e2187a`; command implementation commit: `9726ed58cc1581217f43f23e38d5c1e026208381`.
- Bootstrap foundation: `d3bcb67e927e0cca4b146185f54926595a53d343`; initial cutover release: `e87c3237db0e256fb4af225c93ab6e4fd4660a67`.
- Immutable updater implementation: `d360c1f33e19077e5f09925dc97ff05617109e57`; trusted-command-symlink fix and current committed HEAD before this ledger update: `7a531a109b9ee0212961493513b3145531e2948e`; base was `7323fa42ad4c118f94abf2e58e60ca120ded7207`; branch `main` is 34 commits ahead of `origin/main`. Menu implementation and deployed product release remain `b6d22115f1726aeb570e91900cabcc008ca55689`.
- Obsolete inventory deletion: `ba880aefca5e9eec05c00788adc754db8fb03ebd`; heartbeat source: `078aa1ae297f599cbbff75b4efc1cb9f9a510c4e`; config repair: `9ccce385cc860aee48c44e319db7868e39b9d802`; durable-identity repair: `1a8a13fbabdda0c5610d9948cb8855c0417b907c`; deployed verification commit: `358b27d4bc907b7d606a64447fae064e44e2187a`.
- Initial and final menu reviews are `tmp/agent/menu-reduction-review.md` and `tmp/agent/menu-reduction-final-review.md`; the initial findings were repaired and final review found no blocker. Build/test evidence is under `tmp/agent/menu-reduction-20260814T101014Z/`; backup, deployment, restart, ping, and boundary evidence is under `tmp/agent/menu-deploy-20260814T104000Z/`. Prior exact-`358b27d` live evidence remains under `tmp/agent/heartbeat-adoption-20260814T091913Z/`.
- Initial deployment evidence remains under `tmp/agent/deploy-20260814T041811Z/`; command release evidence is under `tmp/agent/velocity-command-20260814T070259Z/` via `tmp/agent/velocity-command-latest`.
- Exact deployed private release: `~/lkjmc-private-releases/b6d22115f1726aeb570e91900cabcc008ca55689/`; bundle: `~/lkjmc-private-releases/release-b6d22115f1726aeb570e91900cabcc008ca55689.tar.gz`.
- First updater release `31de8b976615f749fec9c1d68187e098b1b52704` has 14 artifacts, manifest SHA-256 `64d8a0bb7865e22454445d6910870238a91bf452d4af216c2b842beacec18315`, and bundle SHA-256 `e3128d01bd87830716d519f491d2f81d43231aef82f115ee525071ccbe792003`. Disposable unprivileged clone `lkjmc-update-drill-20260814T124418Z` has no proxy device, uses DHCP `10.161.59.208`, and has rollback snapshot `pre-immutable-update-20260814T124451Z`. Exact preflight stopped on the trusted-command symlink check before backup, fence, service stop, pointer change, migration, or plugin change; cloned `b6d2211` remained active and healthy.
- Bundle SHA-256: `c485816f31de601d4debd95805c6efad8ae8457000ff91878be0a0a8df17a9e7`; manifest SHA-256: `6756ab7141c177f02bd9bd538f67bffd96380c1afcc1cc25545d20682948a818`. Artifact hashes are recorded in `tmp/agent/menu-deploy-20260814T104000Z/release-artifacts.sha256`; all release and installed plugin hashes passed after both restarts.

## Confirmed facts

### Implemented and tested

- `bootstrap.apply` is the only admitted external-effect command. It is authorized only for a kernel-verified local Unix peer and has a 20-minute decoded-command budget; request body ingestion, decode, and all ordinary commands retain the 8-second admission deadline.
- Bootstrap deterministically owns Velocity, hub, and survival, exact configured server-asset path/digest/project binding, Stop-before-Render-before-Start fencing, readiness waits, no-op reapply, partial-failure repair, and process adoption/recovery.
- Absent runtimes are distinct from owned runtimes. Persisted identities are removed only when the exact identity no longer matches and the recorded process group is absent; live/reused groups remain fenced.
- Backend `server-ip` is rendered from intent and checked as one canonical effective Java-properties assignment. Velocity bind is checked at the top-level TOML key. Closed-listener `TIME_WAIT` is not mistaken for an unowned listener.
- Database-backed bootstrap coverage passes all nine recovery/apply tests. Focused runtime, transport, property-parser, and unowned-listener regressions pass. Workspace Clippy and `./scripts/verify-fast.sh` pass.
- The exact serving release contains six fresh artifacts. Manifest verification, deployed hash verification, CLI identity, JVM identity, server-jar hashes, and installed plugin-jar comparisons all passed.
- The `d360c1f` updater release contract contains fourteen exact artifacts: three Rust binaries, three JVM jars, deploy/publish/backup/restore/restart/fence-check tools, the canonical systemd unit, and its deployment-fence drop-in. The updater requires the exact packaged deployer under a root-owned non-writable anchored release, parses the service-owned database environment as one URL assignment rather than shell, refuses existing backup destinations, and independently re-lists/extracts the PostgreSQL dump before stop.
- Update/no-op/recovery share a root-owned nonblocking global lock. Changed updates install an effective systemd fence before writing a durable root journal under `/var/lib/private`; reboot remains blocked while a fence exists unless the root updater creates one bounded `/run` start permit. Plugin publication occurs only after every service-user process exits and uses no-follow directory descriptors. A changed or unreadable migration ledger remains fenced and forbids binary-only rollback.
- `tests.test_deploy_release` covers the exact anchored closure, config artifact publication, stable no-op, pre-commit rollback, committed-tree retention, fake backup rejection, data-only environment parsing, lock contention, final-value EULA parsing, privileged one-use fence permits, root execution-input ownership, duplicate topology rejection, migration classification, and installer withdrawal. `./scripts/check-installer.sh`, operations mutations, `./scripts/verify-fast.sh`, and disposable-PostgreSQL-16 `./scripts/verify-full.sh` pass. Evidence is under `tmp/agent/immutable-update-20260814T113148Z/`; reviews are `tmp/agent/immutable-update-review.md`, `tmp/agent/immutable-update-final-review.md`, and `tmp/agent/immutable-update-gate-review.md`. The gate review found no blocker/high issue after repairs. This is deterministic evidence only; the real LXC update boundary remains pending.
- The deployed Velocity command registers one Brigadier root with only `status` and `server <hub|survival>`. Status pings both registered servers asynchronously. Transfer accepts only a real Velocity `Player` and uses the platform connection-request future.
- Status and transfer have three/five-second feedback deadlines plus 8/32-operation admission bounds. A timed-out original Velocity operation retains its permit until it actually settles. Shutdown closes feedback synchronously before queued unregister/runtime cleanup.
- Focused command tests execute root/subcommand completion, both status probes, success, all failure statuses, exceptional failure, timeout feedback, pending-future immediate return, ninth/33rd rejection, settlement-driven permit release, close suppression, and 100 lifecycle replacement cycles. Independent follow-up review found no remaining command-slice Java source blocker.
- The heartbeat source uses an empty-body loopback endpoint whose identity comes only from one exact-scope instance credential. Paper/Folia and Velocity cannot select another instance or invoke generic commands. Presence and the two fixed proxy registrations commit atomically.
- Three Java reporters run one fixed-delay operation each off platform threads, re-read mode-`0600` credentials from a mode-`0700` directory, retry after outage, emit transition-only secret-free diagnostics, and are interrupted and joined by the runtime lifecycle deadline.
- Velocity verifies exact `hub=127.0.0.1:25566` and `survival=127.0.0.1:25567` registrations before emitting its success diagnostic or starting heartbeat; a correct-name/wrong-port lifecycle test fails closed and removes all installed surfaces.
- Java child launch now clears the daemon environment before applying instance-scoped values, preventing inherited PostgreSQL or bootstrap credentials. Migration 52 only widens the token principal-kind constraint to `instance`.
- The reduced Paper/Folia bundle is closed to `root`, `docs-directory`, `docs-file`, `docs-links`, and `docs-search`. Its only authored actions are inert command guidance and docs navigation; renderer chrome adds only Back and Close. Remote snapshot views, refresh, pending responses, confirmation, mutation actions, and generic bodies were removed from the menu engine and shaded-jar surface.
- Dynamic probes traverse directory → file → links → linked file, verify direct-route Back parameter projection, ensure a failed parameter check does not mutate session state, compare localized goldens, and reject removed classes in the jar.
- Migration 53 removes the unused `menus` sync domain, its four triggers, active/archive rows, PostgreSQL domain allowance, Rust payload reader, JVM decoder/domain/generated records, and shaded classes. Fresh and version-51 upgrade tests pass against real PostgreSQL; removal creates an explicit feed reload boundary rather than exposing an unknown domain.
- The obsolete regex/source-symbol data-workflow inventory was deleted. Focused real-PostgreSQL rollback, crash, lock-deadline, migration, replay, and terminal-row tests are the executable transaction evidence.
- `./scripts/verify-fast.sh`, `./gradlew :platforms:jvm:paper:check --no-daemon`, and a fresh `LKJMC_STORE_TEST_DATABASE_URL=<disposable-postgresql-16> ./scripts/verify-full.sh` pass for exact commit `b6d2211`, including real migration-53, six-domain sync, dynamic menu graph, generated bindings, shaded jars, and containment checks. Only explicitly guarded jar-live, claim, web, installer, and plugin-asset smokes were skipped.

### Deployed and observed

- Serving container: `lkjmc-next`, Ubuntu Noble image fingerprint `343e93956ae55eca4ce8846d45d61657a3062de131861ab6ac31bddfe21e4cec`, IP `10.161.59.8`, 6 CPUs, 16 GiB memory limit, 128 GiB ZFS root, `security.nesting=false`, unprivileged idmap.
- Runtime packages: PostgreSQL 16 and OpenJDK 21. The live database now has 52 applied migrations through version 53; removed version 15 remains absent. Migration 53 removed the `menus` revision/feed/archive rows, four menu-sync triggers, and that domain from the check constraint; migration 52 remains the instance-principal change.
- systemd starts the exact release and runs local bootstrap reapply through `ExecStartPost`. `KillMode=mixed` permits daemon-owned graceful Java shutdown. A bounded pre-apply plan retry tolerates only transient stale/unowned observations; other blocked plans fail startup.
- Clean first installation and first start succeeded without an intermediate repair: systemd active, `NRestarts=0`, one daemon, Velocity, hub, and survival.
- Velocity listens on `0.0.0.0:25591`; hub and survival listen only on loopback ports `25566` and `25567`; daemon HTTP `8765` and PostgreSQL `5432` are loopback-only.
- Real Minecraft status pings observed Velocity `3.4.0-SNAPSHOT`/protocol 774 and Folia `1.21.11-14` for both backends. Host access to backend, daemon HTTP, and PostgreSQL ports fails.
- No-op apply preserves all three Java PIDs.
- A direct `systemctl restart lkjmc-daemon` replaced all three PIDs, returned success, and ended with a no-op plan and `NRestarts=0`.
- A full Incus container restart changed boot ID, restored the daemon and three Java children, returned a no-op plan, and left `NRestarts=0`.
- Public cutover moved only TCP `25591` to `lkjmc-next`. Legacy daemon TCP `18765` and unused Bedrock UDP `25592` devices were removed, not moved.
- Historical `lkjmc`, intermediate `lkjmc-candidate`, and preserved `lkjmc-next-legacy-20260814` are stopped. Only `lkjmc-next` is running and only it owns a proxy device.
- Host-LAN Java ping still succeeds after stopping both prior serving candidates. External `api.mcstatus.io` reports `lkjsxc.com:25591` online with MOTD `lkjmc network`, zero players, and Velocity `1.7.2-1.21.11` compatibility text.
- Current CLI status at `b6d2211` reports hub and survival `processHealthy:true`, `ready:true`, `joinable:true`, canonical loopback routes, and fresh heartbeat/proxy-registration ages. Velocity remains correctly non-backend with readiness unavailable. The installed Paper jar reports exactly five routes, authored actions `NAVIGATE,NONE`, six curated docs, and zero removed menu classes.
- The `2a250f0` update stopped all owned processes, atomically replaced both plugin kinds and the exact release pointer, updated systemd to the new daemon, and started successfully without rollback. Velocity logged the exact commit and verified that its command manager retained `/lkjmc`.
- The first `30f27e5` candidate exposed a false config no-op and rolled back. A staged `9ccce38` attempt first found deployment-script socket/start-limit races, then exposed stale durable identity re-adoption; each attempt failed closed and restored healthy `2a250f0` with no instance credentials. Commit `1a8a13f` re-observes an adopted identity before planning and retains PID-reuse fencing. Direct `358b27d` deployment then rendered schema-version-2 config, started all processes, created exactly three scoped credentials, and reached fresh heartbeat.
- Making all three credential files unreadable left all PIDs unchanged while hub and survival became `ready:false`, `joinable:false`, reason `heartbeat-stale`, and proxy registration stale. Restoring mode `0600` produced one active transition per reporter and returned both backends to joinable without PID changes.
- Post-heartbeat systemd restart at `358b27d` replaced all four PIDs in 10 seconds; its Incus restart took 13 seconds. After menu cutover, exact `b6d2211` replaced all four PIDs, applied migration 53, and reached fresh heartbeat. Its systemd restart replaced all PIDs in 9 seconds; its Incus restart changed boot ID and all PIDs in 15 seconds. Every run ended active with `NRestarts=0`, a no-op bootstrap plan, and three protocol pings; external status remained online.

## Preservation, backup, and rollback

- Original verified backup remains at `home-incus:~/backups/lkjmc/pre-recovery-20260813T132523Z/`; manifest SHA-256 `768a121d4138aa7ecd212ab3d7d53a359a12ece2dd30270a8b44e913d384c282`.
- Historical snapshots include `pre-recovery-20260813T132126Z`, `pre-latest-rebuild-20260814T042229Z`, and the fresh pre-cutover snapshot named in `tmp/agent/deploy-latest/historical-pre-cutover-snapshot-name.txt`.
- The old stopped `lkjmc-next` was renamed to `lkjmc-next-legacy-20260814`; its two original snapshots remain intact. Its current NIC override was removed only to release static IP `.8`; snapshots retain the old configuration.
- Final cutover snapshots are named in `lkjmc-next-pre-container-restart-snapshot-name.txt`, `lkjmc-next-validated-snapshot-name.txt`, and `lkjmc-next-post-cutover-snapshot-name.txt`.
- Command-update rollback snapshots are named in `tmp/agent/velocity-command-latest/pre-command-snapshot-name.txt` and `post-command-snapshot-name.txt`. Heartbeat boundaries are `pre-heartbeat-20260814T084141Z`, `pre-heartbeat-repair-20260814T090824Z`, and `post-heartbeat-20260814T093801Z`. Exact `2a250f0` and `e87c323` releases remain installed for executable rollback.
- Fresh pre-migration database backup: `home-incus:~/backups/lkjmc/pre-menu-reduction-20260814T105555Z/`, with mode `0700` directory and mode `0600` members. All checksums passed; restore into an empty owner-correct database returned `51|52` migrations, 3 instances, and 2 jar assets before cleanup. Incus rollback snapshots are `pre-menu-reduction-20260814T105555Z` and `post-menu-reduction-20260814T110256Z`.
- Earlier serving backup remains `home-incus:~/backups/lkjmc/e87-cutover-20260814T063433Z/private-backup-20260814T063433Z/`. Its PostgreSQL dump and filesystem archive already passed the recorded restore/extraction drill.
- Migration 53 makes the old `358b27d` binary's migration ledger incompatible with the current database. Do not roll back only the binary. Restore Incus snapshot `pre-menu-reduction-20260814T105555Z`, or restore the verified pre-menu database into a fresh target together with the matching old release. Fast traffic rollback to historical `lkjmc` remains separate and must move only TCP `25591`.

## Decisions

- Historical world and database state may be discarded; backups and snapshots are the rollback boundary.
- Incus remains authoritative. No Docker socket, Incus socket, host mount, bridge bypass, privileged mode, or nested container support was added.
- Keep only the Velocity TCP proxy public. PostgreSQL, daemon HTTP, and backends remain private.
- Existing observed `eula=true` was reused; no acceptance was fabricated.
- Exact source and artifact identity take precedence over installation convenience. Deployment consumed only a clean commit release and verified immutable server jars.
- systemd owns restart reapply because daemon shutdown intentionally stops Java children and daemon startup alone does not reconcile them.
- The broad project is not player-accepted merely because the three-process network is serving.

## Acceptance completed

- Clean install into an unprivileged LXC system container: complete.
- Fresh PostgreSQL migration: complete.
- Exact Velocity plus two exact Folia processes ready: complete.
- Local, host-LAN, and external proxy status ping: complete.
- Backends/daemon/database private-boundary checks: complete.
- Initial apply and PID-preserving no-op reapply: complete.
- systemd restart recovery and full container restart recovery: complete.
- Private backup, checksum verification, PostgreSQL restore, and filesystem extraction drill: complete.
- Public TCP cutover with old deployment stopped and rollback retained: complete.
- Velocity `/lkjmc` source implementation, focused deterministic tests, exact release, atomic update, live command-manager registration, systemd restart, and container restart: complete. Minecraft-client command/completion/transfer observation is not complete.
- Heartbeat endpoint/reporter, exact credential policy and route verification, environment isolation, fresh/stale availability, proxy-registration rollback, request/database deadlines, migration-51 upgrade, legacy-config and durable-identity repair, independent review, complete disposable-PostgreSQL verification, exact release, migration, credentialing, stale/recovery drill, systemd restart, Incus restart, protocol pings, and private boundaries: complete and live-observed at `358b27d`.
- Five-route local menu source, dynamic navigation/Back behavior, locale goldens, removed menu snapshot/mutation classes, migration 53, six-domain sync, exact release, atomic deployment, systemd restart, Incus restart, pings, and boundaries: implemented, tested, deployed, and process-observed at `b6d2211`; not player-observed.

## Current failures and blockers

- The serving `b6d2211` Velocity registers `/lkjmc`, but no authorized real client has observed command parsing, completion, status text, successful transfer, or failed-transfer feedback.
- No authorized real online-mode player/client login has run. The deployment is serving and externally pingable, but not player-accepted.
- The reduced menu is deployed and jar-observed, but no authorized player has opened `/menu` or `/docs`; rendered probe frames are not player acceptance.
- Clean installation remains unsupported. `scripts/install.sh` exits before mutation instead of building ambient checkout bytes. The immutable existing-deployment updater has deterministic evidence; its first disposable-LXC preflight failed closed on Ubuntu command symlinks and the fix is committed, but the changed update/no-op/restart/restore acceptance remains incomplete.
- Incus global container drop-ins override some systemd hardening such as `NoNewPrivileges`; the service remains an unprivileged `lkjmc` user inside an unprivileged container, with strict filesystem write paths and no public control API.
- Git push/release publication authentication remains unchecked.

## Exact verification commands

```sh
cargo test -p lkjmc-core network_intent_tests -- --nocapture
cargo test -p lkjmc-daemon transport:: -- --nocapture
cargo test -p lkjmc-daemon runtime::local::tests -- --nocapture
cargo test -p lkjmc-daemon rendered_file_tests -- --nocapture
LKJMC_STORE_TEST_DATABASE_URL=<disposable-postgresql-url> cargo test -p lkjmc-daemon commands::bootstrap_api::apply::network_probe_tests -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
./scripts/verify-fast.sh
./scripts/check-installer.sh
python3 scripts/check-operations.py --all --mutations
./gradlew --no-daemon --no-build-cache :platforms:jvm:velocity:test
./gradlew --no-daemon --no-build-cache :platforms:jvm:common:test --tests com.lkjmc.common.heartbeat.PluginHeartbeatReporterTest
LKJMC_STORE_TEST_DATABASE_URL=<disposable-postgresql-url> cargo test -p lkjmc-daemon transport::heartbeat::tests -- --nocapture
LKJMC_STORE_TEST_DATABASE_URL=<disposable-postgresql-url> cargo test -p lkjmc-store --test safety plugin_heartbeat_identity_upgrade_preserves_version_51_credentials -- --exact --nocapture
python3 tests/test_data_workflow_checker.py
LKJMC_STORE_TEST_DATABASE_URL=<disposable-postgresql-url> ./scripts/verify-full.sh
python3 scripts/check-jvm-containment.py
./gradlew --no-daemon :platforms:jvm:paper:check
LKJMC_STORE_TEST_DATABASE_URL=<disposable-postgresql-16> ./scripts/verify-full.sh
./scripts/build-release.sh "$HOME/lkjmc-private-releases/b6d22115f1726aeb570e91900cabcc008ca55689"
./scripts/verify-artifact-manifest.py --manifest "$HOME/lkjmc-private-releases/b6d22115f1726aeb570e91900cabcc008ca55689/artifact-manifest.json" --release-root "$HOME/lkjmc-private-releases/b6d22115f1726aeb570e91900cabcc008ca55689"
ssh home-incus 'incus exec lkjmc-next -- systemctl restart lkjmc-daemon.service'
ssh home-incus 'incus restart lkjmc-next --timeout 180'
python3 tmp/agent/deploy-20260814T041811Z/minecraft-status.py lkjsxc.com 25591
```

The direct local public-DNS ping times out because this workspace cannot hairpin through the router; the host-LAN protocol ping and an independent external Minecraft status provider both pass.

## Next executable step

Build the exact private fourteen-artifact `7a531a109b9ee0212961493513b3145531e2948e` release, then execute `b6d2211` → `7a531a1` plus identical no-op, systemd restart, Incus restart, backup verification/restore, hashes, pings, heartbeat, fence/permit behavior, and private-boundary checks in a disposable unprivileged clone. Deploy to `lkjmc-next` only after that drill passes and a fresh production snapshot exists. When an authorized account is available, exercise `/lkjmc` completion/status/success/failure transfer and `/menu` docs navigation.
