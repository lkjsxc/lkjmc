# lkjmc

A Minecraft community built around lasting worlds, friends, and temporary
Expeditions. The player experience provides Play, Worlds, People, and Timeline;
Hosting is a separate workspace for server owners and operators.

The application consists of a Rust Core, PostgreSQL, a React Web client,
Velocity/Paper adapters, and an Incus host agent. Core owns authorization and
transactions; adapters own observable game effects. World and inventory changes
use durable receipts so interrupted work can be reconciled without duplication.

[GitHub lkjsxc/lkjmc](https://github.com/lkjsxc/lkjmc) is the application source.
Forgejo `lkjsxc/lkjmc-rebuild` verifies the same commit and retains release
artifacts. Forgejo GitOps owns production deployment. Existing accounts, worlds,
assets, and recovery records are preserved through updates.

- [Navigation and languages](docs/languages-and-ui.md)
- [Interaction and recovery contracts](docs/ux-contract.md)
- [Current implementation and verification](docs/work/active.md)
- [Acceptance boundaries](docs/acceptance.md)

## Development and verification

Use Rust, Node, JDK 25, PostgreSQL 18, and matching `pg_dump`/`pg_restore`.
Use the pinned versions in `ops/ci/toolchains.lock.json` and the existing isolated
CI environment for a complete release check.

`python3 scripts/dev.py test --locked --offline` runs database-isolated Rust tests.
The configuration must refer to a development database: tests also create and
remove their own restore databases. Do not initialize a new password against an
existing shared development container. Set `LKJMC_PG_DUMP` to the PostgreSQL 18
binary when it is outside the default installation.

`python3 -m unittest discover -s tests/guest -v` checks guest file and receipt
boundaries. `python3 -m unittest discover -s tests/ci -v` checks locale and source
policies. Web scripts distinguish state, controlled browser fixtures, and real
Core/browser integration. The latter requires an explicitly isolated test account.

`scripts/ux_verify.py --browser --protocol` uses a dedicated loopback database and
actual Paper/Velocity processes. Its offline Java clients do not establish
licensed Java authentication, real Bedrock acceptance, or external connectivity.
Never reset another task's development worlds to run it.

Official automatic backups default to 18:00 UTC (03:00 in Japan). They retain
seven daily and four weekly successful backups; manual and pinned backups are
not automatically pruned. Development mode disables automatic backups.
`LKJMC_AUTOMATIC_BACKUPS=false` disables scheduling explicitly.
