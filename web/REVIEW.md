Player-first frontend review
============================

The player workspace has four destinations: Play, Worlds, People, and Timeline. Hosting is a separate workspace and Account stays at the bottom of navigation. Hash routes use one canonical vocabulary; retired routes do not redirect. Worlds groups land, homes, achievements, storage, and economy tools around an actual world. Expeditions replaces the public Private End/adventure terminology with an explicitly temporary, participant-only experience.

Play and Hosting consume authoritative Core projections. A Minecraft connection address never requires a linked web game identity; browser travel requires the projected identity/session and `status.actions.join` capability. Status badges use typed Minecraft state and unknown player counts stay unknown when observations are stale. Host and Minecraft phases remain separate. Durable jobs expose receipt-backed completion, and uncertain delivery keeps the admitted operation rather than submitting a replacement.

Expeditions lives inside lkjmcsmp at `/worlds/{serverId}/expeditions`, with nested journal and detail routes. Worlds has no global Expeditions tab. Direct links first load and validate the authorized official server; tabs, breadcrumbs, titles, pagination, and return links keep that server context. Expedition prices and lifetime come from the server. The preparation roster, wallet, and `can_prepare` capability govern the preparation review. Current expeditions expose independent enter/return/cancel/refund actions. The journal uses participant-scoped, paginated `/api/v1/expeditions` reads and authorized detail routes; it creates no world archives or download promise. Cursor navigation uses the same identity, abort, and authorization lifecycle as the rest of the application.

Product language uses stable message IDs and structured system messages. UI errors, open dialog labels/options/notes, operation titles, and toasts render in the current language. Generated asset, achievement, and rank titles use explicit provenance messages; player names, chat, custom titles, and file drafts remain verbatim.

Verification lanes
==================

- `npm run build` type-checks and builds production source. `npm run test:state` runs the state, real API boundary, canonical-route, and English/Japanese catalog tests. No service is needed.
- `npm run test:fixture` runs browser contract fixtures at `https://ux.fixture`; other origins are blocked. This includes player journeys, preparation/journal, Hosting permissions, conversation consistency, and retained session/file regressions. Widths 320/360/390/768/1024/1440 run in English and Japanese. Build first.
- `npm run test:integration` runs real Core journeys only against the dedicated loopback test service at `http://127.0.0.1:18091`. It requires `.local/browser-session.json` relative to the repository or `LKJMC_TEST_SESSION`, with a dedicated test account's token. Missing/expired credentials fail explicitly. This lane changes language and creates a group, message, and report; it performs no production writes.
- `npm run test:e2e -- --list` discovers both projects without reading credentials or contacting the service. Fixture screenshots in `.local/player-first` illustrate the rendered interface; they are not integration or production evidence.

Session and retention boundaries
================================

Account or CSRF changes, API 401, and successful logout invalidate the identity epoch, abort old work, clear private caches, and unmount private UI state. Same-session refresh preserves forms and focus. Account changes can be recognized only from server responses; `/me` refresh runs on window focus and every 15 seconds.

403/404 remove current resource contents and controls. Transport/5xx failures can retain visibly marked last-known state. Reopened reads hide previous output and reauthorize. Missing ephemeral jobs discard the job/key and require explicit Retry. Uncertain submissions and transient status failures retain their original key/job. File drafts require a current successful read on reopen; ordinary status changes preserve them.

Timeline keeps at most 8 windows of 200 items and 32 conversation drafts, and prunes rooms against current membership. File drafts keep at most 16 entries, scoped reads at most 24 entries, and read results expire after two minutes. Console drafts and directory/date choices are bounded and session-reset. All updates and Activity are read-only; selecting a conversation binds its read window and composer to the same recipient.
