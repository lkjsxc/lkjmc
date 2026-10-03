Frontend review lanes
=====================

- `npm run build` builds the actual source. `npm run test:state` type-checks the ReadSession suite and runs state/API tests with Node's required `--experimental-vm-modules` flag. No service is needed.
- `npm run test:fixture` (or `node ../scripts/check_ui.mjs --fixture`) runs frontend contract fixtures. All traffic is fulfilled at `https://ux.fixture`; other origins are blocked. Includes 320/360/390/768/1024/1440 widths in English and Japanese. Build first.
- `npm run test:integration` (or the default `node ../scripts/check_ui.mjs`) runs the real journeys against the dedicated local test service at `http://127.0.0.1:18091`. It requires `.local/browser-session.json` relative to the repository, or `LKJMC_TEST_SESSION`, containing a dedicated test account's `token`. A missing/expired session is an error; no silent skips or mock fallback. This lane changes language, creates a group/message/report, and requires explicit authorization to run against that dedicated service.
- Default `npm run test:e2e` discovers both projects; `--list` discovers real journeys without reading credentials or contacting the service. The live lane was not run in this review.

Unresolved release gates
========================

Timeline's backend is absent. The fixture's Timeline response is a frontend contract, including optional comma-separated `known_ids` refresh input and `updates`/`removed_ids` response fields, plus current `rooms` membership. It is not backend implementation or integration evidence. No server-specific Timeline filter has been invented: legacy server activity bookmarks go to server status; Home activity bookmarks go to Timeline events. Durable job progress/detail and building preview/consent remain.

The known host candidate refuses sleeping-VM READs and all plugins/config paths. The success fixtures model frontend file/read/write contracts on synthetic `documents/notes.txt` paths; they do not prove host usability, stopped-server file editing, or plugin/config support. A separate refusal fixture verifies that a host denial stays an error, clears content, disables actions, and does not retry automatically. Resolving the host gate requires separately authorized backend/host work, not relaxed permissions or frontend workaround.

Session and retention boundaries
================================

`/me` account or CSRF changes, any API 401, and successful logout synchronously invalidate the identity epoch, abort old requests, clear registered caches, and unmount private UI state. Same-session `/me` refresh preserves forms and focus. Identity can only be recognized from server responses; this does not claim to detect an unreported cookie change before the next `/me` check (15 seconds or window focus).

403/404 clear current resource data rather than labeling it stale. Transport/5xx failures can retain visibly marked last-known state. Reopened READs hide previous output and reauthorize; missing ephemeral jobs discard the job and idempotency key, and only explicit Retry admits a new READ. Uncertain submissions and transport status failures keep their existing key/job. File drafts are gated behind a current successful read on reopen; ordinary status changes preserve them.

Timeline retains at most 8 windows of 300 items, 32 conversation drafts, and prunes rooms from the current membership response. File drafts retain at most 16 entries; reads at most 24 scopes, with a two-minute result retention bound. Console drafts and directory/date choices are also bounded and session-reset. These bounds deliberately limit scrollback retained across navigation. Browser fixture execution must be completed in the coordinator's authorized browser environment; sandbox launch failure is not a passing test.
