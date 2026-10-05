# Player experience and language boundaries

The source implements a player-first experience; deployment and runtime
acceptance are recorded separately in `docs/work/active.md`.

## Navigation

Play answers where to play next, who is available, and how to connect. Resume
selection considers the current destination and confirmed travel history only
while access remains valid. Presence respects friendship, both directions of
blocks, activity privacy, and private-server access. A pending friend request
does not grant presence visibility. The lobby is a connection entrypoint, not a
preferred resume destination.

World pages group tools into World and Economy. Quiet Worlds/Expeditions
navigation replaces the prominent Expedition promotion. People groups Friends,
Teams, and Parties. Teams lists every membership; each team's overview, members,
and settings have a distinct team-ID route.
Timeline's combined updates are read-only; selecting a conversation controls both
the visible history and the message recipient. Hosting provides status, Console,
Logs, Files, Backups, Members, and Settings in a separate workspace. Account
settings remain at the bottom of navigation.

Hosting uses a compact server inventory and a persistent server identity across
its tools. Files uses plain directory rows, breadcrumbs, and a separate action
toolbar. Directory/file selections are URL state. A desktop editor sits beside
the directory at 1024px and above; smaller screens show a dedicated editor.
Admin Operations separates in-progress, failed/uncertain, and terminal history;
ordinary queued or running work is not an attention warning.

Team permissions remain independent across memberships. The contribution team
receives automatic shared progress and rewards; it is never an implicit wallet
or permission context. Players can clear that preference. Additional memberships
do not change it. New teams earn land capacity through achievements, while
existing team assets and capacity are preserved.

Routes preserve resource identities and selections in the URL. Old navigation
aliases are retired during the coordinated release. Authorization always remains
in Core, including read endpoints, history, job details, and file operations.

The visual system has one set of tokens and shared components. Graphite surfaces,
neutral text, emerald primary actions, and violet Expedition accents communicate
hierarchy without making color the only state indicator. Layouts support 320px
screens, visible keyboard focus, large touch targets, and reduced motion.
Background refresh preserves same-session drafts, focus, and scroll; identity
changes and permission revocation remove private data.

## Expeditions

An Expedition is a temporary world. Its environment, participant access, lifetime,
and compute availability are independent concepts. The initial destination is
The End: 1,000 coins and 12 Eyes of Ender, lasting three hours from activation.
Core supplies the displayed cost, duration, preparation requirements, and actions.

Participants commit at preparation. Changing a party or its ready flags later
does not change committed Expedition membership. Entering is explicit. Returning
uses a durable origin saved before entry, with a valid bed or established safe
SMP position as fallback. Returning does not give up the right to re-enter before
expiry. Moderation and active game-session requirements still apply.

Carried inventory survives. Blocks, containers, dropped items, and the temporary
world are removed at closure. Evacuation and recoverable return obligations precede
world deletion. Failed preparation and cancellation retain exactly-once refund
settlement. The participant-visible journal retains dates, roster, and outcome;
it does not promise a downloadable world or generated screenshots.

## Language

`locales/languages.json` selects English by default and declares supported locales.
English and Japanese catalogs contain the same stable message IDs and parameter
contracts. Web and Minecraft adapters package the same definitions.

System content uses `{id, params}`. Errors, notifications, operation progress,
command help, and generated asset titles are rendered in the selected language
at the presentation boundary. Open dialogs and errors must change with that
selection; language state is not inferred from a browser or Minecraft client.
Unknown messages show a localized explanation and reference instead of raw prose
from another language.

An anonymous visitor's explicit choice is stored locally. After sign-in, the
account preference is authoritative and is shared with Paper/Velocity. User
names, chat, custom titles, raw files, and raw console output remain verbatim.
Native language names in the selector are intentional. `assets.title_message`
is present only for identified system-authored titles; a null value preserves the
user-authored `title`. Seeded achievements and ranks follow the same provenance
rule through `title_message`, `description_message`, and `name_message`.

`scripts/migrate_messages.py` is an offline authoring tool for adopting stable
IDs; it is not a runtime translation fallback. Catalog checks and browser/game
acceptance cover both languages, dynamic system content, and language changes.
