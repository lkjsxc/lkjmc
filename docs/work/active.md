# Player-first lkjmc

Implementation base: `3e7af86a54902188898bcc04ef761f81a7f8a28a`.
Integration branch: `work/player-first-20261004`.

The accepted direction is a player-first community with Play, Worlds, People,
Timeline, a separate Hosting workspace, and temporary End Expeditions. English
is the default and Japanese is a complete separately rendered locale. Existing
worlds, identities, ownership, balances, and physical-operation receipts remain
valuable data; old UI and public API compatibility is not required.

Implementation lanes:

- Runtime: honest uncertain console outcomes, bounded independent passive reads,
  fenced worker ownership, typed machine/game/operation status.
- Expeditions: committed membership, durable safe return, explicit entry/return,
  participant-visible history, and existing refund/deletion guarantees.
- Languages: shared stable message IDs and typed parameters, runtime rendering,
  complete catalogs, provenance-based historical system-message migration.
- Web: new player shell and visual system, direct journeys, conversation-bound
  composition, accessible responsive Hosting tools.
- Game: contextual menus and matching terminology without weakening click,
  session, scheduler, or inventory safeguards.

Verification must distinguish source/unit/fixture coverage from PostgreSQL,
actual Paper/Velocity, Incus, production, licensed Java, and real Bedrock.
Current baseline: 17 Web state/API tests and catalog check passed during planning;
the served production frontend was freshly matched to the base release.
Licensed Java/real Bedrock and external WAN acceptance remain unverified.

Next: integrate independently committed lanes, run the full isolated candidate
checks, review actual screenshots, then package and deploy only through the
existing source/CI/GitOps gates after draining active Expeditions and preserving
recoverable state. Do not import dirty historical handoffs over current source.
