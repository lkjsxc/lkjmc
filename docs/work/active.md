# Active work

## Current objective

The exact `2a250f0b27f3f6680249a16be33f4dfa06e6ed16` release still serves the network after an automatic candidate rollback. The first heartbeat deployment applied migration 52 but exposed a false bootstrap no-op that retained legacy Java environment; no credentials were created and rollback restored all old executable/plugin bytes. Source and exact private release `9ccce385cc860aee48c44e319db7868e39b9d802` now force a fenced legacy-config repair, have no review blocker, and pass the complete Rust/JVM/PostgreSQL/contract/containment gate. The current gate is controlled redeployment, three credentials, and live freshness/staleness/reconnect evidence. Real-client command/completion/transfer remains blocked on an authorized online-mode account.

## Repository state

- Recovery base: `339adb7bb60b6cce5229f98866f869363e90b78b`.
- Product and deployed release commit: `2a250f0b27f3f6680249a16be33f4dfa06e6ed16`; command implementation commit: `9726ed58cc1581217f43f23e38d5c1e026208381`.
- Bootstrap foundation: `d3bcb67e927e0cca4b146185f54926595a53d343`; initial cutover release: `e87c3237db0e256fb4af225c93ab6e4fd4660a67`.
- Deployment ledger checkpoint before this update: `216bdf92b807aeb2fa37c61ae0ba690ae28852d4`.
- Obsolete inventory deletion commit: `ba880aefca5e9eec05c00788adc754db8fb03ebd`; heartbeat source commit: `078aa1ae297f599cbbff75b4efc1cb9f9a510c4e`; live-found configuration repair: `9ccce385cc860aee48c44e319db7868e39b9d802`. Branch `main` tracks `origin/main` and is 24 commits ahead before this ledger checkpoint.
- Final initial review is `tmp/agent/heartbeat-review-final.md`; the live-finding review is `tmp/agent/heartbeat-live-finding-review.md`. Complete exact-`9ccce38` disposable-PostgreSQL gate and release evidence is under `tmp/agent/heartbeat-repair-20260814T085751Z/` via `tmp/agent/heartbeat-latest`; the failed candidate and successful rollback evidence remains under `tmp/agent/heartbeat-20260814T083113Z/`.
- Initial deployment evidence remains under `tmp/agent/deploy-20260814T041811Z/`; command release evidence is under `tmp/agent/velocity-command-20260814T070259Z/` via `tmp/agent/velocity-command-latest`.
- Exact deployed private release: `~/lkjmc-private-releases/2a250f0b27f3f6680249a16be33f4dfa06e6ed16/`. Exact repair candidate: `~/lkjmc-private-releases/9ccce385cc860aee48c44e319db7868e39b9d802/`.
- Repair release archive SHA-256: `3a5ff5f3965e6432c27857a5ad72d91cea11c31933f6786eaef219916d8cb747`; its six fresh artifact hashes are recorded in the repair evidence directory.

## Confirmed facts

### Implemented and tested

- `bootstrap.apply` is the only admitted external-effect command. It is authorized only for a kernel-verified local Unix peer and has a 20-minute decoded-command budget; request body ingestion, decode, and all ordinary commands retain the 8-second admission deadline.
- Bootstrap deterministically owns Velocity, hub, and survival, exact configured server-asset path/digest/project binding, Stop-before-Render-before-Start fencing, readiness waits, no-op reapply, partial-failure repair, and process adoption/recovery.
- Absent runtimes are distinct from owned runtimes. Persisted identities are removed only when the exact identity no longer matches and the recorded process group is absent; live/reused groups remain fenced.
- Backend `server-ip` is rendered from intent and checked as one canonical effective Java-properties assignment. Velocity bind is checked at the top-level TOML key. Closed-listener `TIME_WAIT` is not mistaken for an unowned listener.
- Database-backed bootstrap coverage passes all nine recovery/apply tests. Focused runtime, transport, property-parser, and unowned-listener regressions pass. Workspace Clippy and `./scripts/verify-fast.sh` pass.
- The exact serving release contains six fresh artifacts. Manifest verification, deployed hash verification, CLI identity, JVM identity, server-jar hashes, and installed plugin-jar comparisons all passed.
- The deployed Velocity command registers one Brigadier root with only `status` and `server <hub|survival>`. Status pings both registered servers asynchronously. Transfer accepts only a real Velocity `Player` and uses the platform connection-request future.
- Status and transfer have three/five-second feedback deadlines plus 8/32-operation admission bounds. A timed-out original Velocity operation retains its permit until it actually settles. Shutdown closes feedback synchronously before queued unregister/runtime cleanup.
- Focused command tests execute root/subcommand completion, both status probes, success, all failure statuses, exceptional failure, timeout feedback, pending-future immediate return, ninth/33rd rejection, settlement-driven permit release, close suppression, and 100 lifecycle replacement cycles. Independent follow-up review found no remaining command-slice Java source blocker.
- The heartbeat source uses an empty-body loopback endpoint whose identity comes only from one exact-scope instance credential. Paper/Folia and Velocity cannot select another instance or invoke generic commands. Presence and the two fixed proxy registrations commit atomically.
- Three Java reporters run one fixed-delay operation each off platform threads, re-read mode-`0600` credentials from a mode-`0700` directory, retry after outage, emit transition-only secret-free diagnostics, and are interrupted and joined by the runtime lifecycle deadline.
- Velocity verifies exact `hub=127.0.0.1:25566` and `survival=127.0.0.1:25567` registrations before emitting its success diagnostic or starting heartbeat; a correct-name/wrong-port lifecycle test fails closed and removes all installed surfaces.
- Java child launch now clears the daemon environment before applying instance-scoped values, preventing inherited PostgreSQL or bootstrap credentials. Migration 52 only widens the token principal-kind constraint to `instance`.
- The obsolete regex/source-symbol data-workflow inventory was deleted. Focused real-PostgreSQL rollback, crash, lock-deadline, migration, replay, and terminal-row tests are the executable transaction evidence.
- `./scripts/verify-fast.sh` and a fresh `LKJMC_STORE_TEST_DATABASE_URL=<disposable-postgresql-16> ./scripts/verify-full.sh` pass at exact repair commit `9ccce38`. The full gate includes workspace formatting, Clippy with `-D warnings`, all Rust/JVM tests, database probes, generated bindings, shaded jars, and artifact containment; only explicitly guarded live smokes were skipped.

### Deployed and observed

- Serving container: `lkjmc-next`, Ubuntu Noble image fingerprint `343e93956ae55eca4ce8846d45d61657a3062de131861ab6ac31bddfe21e4cec`, IP `10.161.59.8`, 6 CPUs, 16 GiB memory limit, 128 GiB ZFS root, `security.nesting=false`, unprivileged idmap.
- Runtime packages: PostgreSQL 16 and OpenJDK 21. The live database now has 51 applied migrations through version 52; removed version 15 remains absent. Migration 52 preserved the preexisting credential count and only widened principal kind for scoped instances.
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
- Current CLI status truthfully reports process health but backend `ready:null`, `joinable:false`, and `heartbeat-missing`; plugin heartbeat/registration is implemented only in the newer working source and is not deployed.
- The `2a250f0` update stopped all owned processes, atomically replaced both plugin kinds and the exact release pointer, updated systemd to the new daemon, and started successfully without rollback. Velocity logged the exact commit and verified that its command manager retained `/lkjmc`.
- The first `30f27e5` heartbeat candidate applied migration 52, then bootstrap incorrectly returned no-op because legacy instance config was not part of observed drift. New plugin lifecycle futures failed before registration because the heartbeat environment was partial. The rollback trap restored `2a250f0`, all three old plugin files, the unit, release pointer, and healthy service. No scoped credentials or secret files were created; migration 52 safely remains.
- Post-update systemd restart replaced all four PIDs in 11 seconds. A full Incus restart changed boot ID and all PIDs in 15 seconds. Both ended active with `NRestarts=0`, a no-op bootstrap plan, command-registration log, three protocol pings, and `boot.autostart=true`.

## Preservation, backup, and rollback

- Original verified backup remains at `home-incus:~/backups/lkjmc/pre-recovery-20260813T132523Z/`; manifest SHA-256 `768a121d4138aa7ecd212ab3d7d53a359a12ece2dd30270a8b44e913d384c282`.
- Historical snapshots include `pre-recovery-20260813T132126Z`, `pre-latest-rebuild-20260814T042229Z`, and the fresh pre-cutover snapshot named in `tmp/agent/deploy-latest/historical-pre-cutover-snapshot-name.txt`.
- The old stopped `lkjmc-next` was renamed to `lkjmc-next-legacy-20260814`; its two original snapshots remain intact. Its current NIC override was removed only to release static IP `.8`; snapshots retain the old configuration.
- Final cutover snapshots are named in `lkjmc-next-pre-container-restart-snapshot-name.txt`, `lkjmc-next-validated-snapshot-name.txt`, and `lkjmc-next-post-cutover-snapshot-name.txt`.
- Command-update rollback snapshots are named in `tmp/agent/velocity-command-latest/pre-command-snapshot-name.txt` and `post-command-snapshot-name.txt`. The heartbeat pre-update snapshot is `pre-heartbeat-20260814T084141Z`. The exact old release and pre-update plugin bytes remain installed for fast rollback.
- Fresh serving backup: `home-incus:~/backups/lkjmc/e87-cutover-20260814T063433Z/private-backup-20260814T063433Z/`, 433,153,045 bytes before the candidate-local copy was removed. All five checksum targets passed.
- The PostgreSQL custom dump restored into a new disposable database and returned `50` migrations, `3` instances, and `2` jar assets. The filesystem archive extracted into a disposable root; both worlds, private config, and exact server assets were verified before deletion.
- Fast traffic rollback: remove `proxy-25591-tcp` from `lkjmc-next`, add the recorded identical TCP proxy device to stopped historical `lkjmc`, start historical `lkjmc`, and protocol-ping the host path. Do not restore daemon HTTP or Bedrock exposure unless explicitly approved.

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
- Heartbeat endpoint/reporter, exact credential policy and route verification, environment isolation, fresh/stale availability, proxy-registration rollback, request/database deadlines, migration-51 upgrade, fresh-schema tests, legacy-config repair, independent review, complete disposable-PostgreSQL verification, and exact private release: complete through `9ccce38`. Migration 52 is live; the repaired release is not yet deployed or credentialed and heartbeat is not live-observed.

## Current failures and blockers

- The serving `2a250f0` Velocity registers `/lkjmc`, but no authorized real client has observed command parsing, completion, status text, successful transfer, or failed-transfer feedback.
- No authorized real online-mode player/client login has run. The deployment is serving and externally pingable, but not player-accepted.
- The serving release still has no plugin heartbeat; status therefore remains `ready:null`/non-joinable until `9ccce38` is deployed, credentialed, and observed.
- The small menu has not been reduced to and proven against real actions.
- The shipped installer is not the deployment path used here; it still builds ambient checkout bytes and does not represent this immutable three-instance installation.
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
./gradlew --no-daemon --no-build-cache :platforms:jvm:velocity:test
./gradlew --no-daemon --no-build-cache :platforms:jvm:common:test --tests com.lkjmc.common.heartbeat.PluginHeartbeatReporterTest
LKJMC_STORE_TEST_DATABASE_URL=<disposable-postgresql-url> cargo test -p lkjmc-daemon transport::heartbeat::tests -- --nocapture
LKJMC_STORE_TEST_DATABASE_URL=<disposable-postgresql-url> cargo test -p lkjmc-store --test safety plugin_heartbeat_identity_upgrade_preserves_version_51_credentials -- --exact --nocapture
python3 tests/test_data_workflow_checker.py
LKJMC_STORE_TEST_DATABASE_URL=<disposable-postgresql-url> ./scripts/verify-full.sh
python3 scripts/check-jvm-containment.py
./scripts/build-release.sh "$HOME/lkjmc-private-releases/2a250f0b27f3f6680249a16be33f4dfa06e6ed16"
./scripts/verify-artifact-manifest.py --manifest "$HOME/lkjmc-private-releases/2a250f0b27f3f6680249a16be33f4dfa06e6ed16/artifact-manifest.json" --release-root "$HOME/lkjmc-private-releases/2a250f0b27f3f6680249a16be33f4dfa06e6ed16"
ssh home-incus 'incus exec lkjmc-next -- systemctl restart lkjmc-daemon.service'
ssh home-incus 'incus restart lkjmc-next --timeout 180'
python3 tmp/agent/deploy-20260814T041811Z/minecraft-status.py lkjsxc.com 25591
```

The direct local public-DNS ping times out because this workspace cannot hairpin through the router; the host-LAN protocol ping and an independent external Minecraft status provider both pass.

## Next executable step

Commit this ledger checkpoint, transfer the exact `9ccce38` release, and first confirm the production bootstrap plan contains ordered Stop/Render/Start repair for all three legacy configs. Deploy through the rollback-capable path, create distinct `proxy`, `hub`, and `survival` credentials without exposing their values, and prove fresh/stale status plus systemd and container restart reconnect. The existing backup was reverified and `pre-heartbeat-20260814T084141Z` is the snapshot boundary. Real command/completion/transfer acceptance remains blocked until an authorized online-mode client is available.
