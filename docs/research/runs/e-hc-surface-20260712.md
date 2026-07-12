# E-HC-SURFACE local run 2026-07-12

## Purpose

Record a local, loopback-only surface comparison. It adds no daemon route,
controller call, extension point, public listener, mobile API, or deployment.

## Identity and environment

Task: `E-HC-SURFACE`; base: `4b9357a8e1a7949e0ebfe59c16af5196554f46cc`;
repaired hypothesis: `efdbdcc`; repaired harness: this branch; seed:
`e-hc-surface-20260712`. The run used local Python 3.12.3. No database,
product daemon, controller, external service, or non-loopback address was used.

## Commands and artifacts

```sh
python3 docs/research/experiments/e-hc-surface/harness.py --self-test \
  --artifact-root /tmp/lkjmc-e-hc-surface-correction-self
python3 docs/research/experiments/e-hc-surface/harness.py --browser-semantic-test \
  --artifact-root /tmp/lkjmc-e-hc-surface-correction-browser
python3 docs/research/experiments/e-hc-surface/harness.py --self-test \
  --mobile-evidence /tmp/lkjmc-e-hc-surface-missing.json
```

The first two commands exited 0 with 12 local cases; the mobile command exited
2 with `mobile=BLOCKED reason=missing-evidence` before a listener or root. Each
successful root retained only `.owned` and `summary.json`. Recursive scans of
both files passed and asserted no bearer, `__pycache__`, or temporary
`third_party_status.py` import artifact. Raw roots are ignored and removable:

```sh
rm -rf -- /tmp/lkjmc-e-hc-surface-correction-self /tmp/lkjmc-e-hc-surface-correction-browser
```

## Observations

The server bound `127.0.0.1:ephemeral`. Its CSP permits `connect-src` only to
its exact `http://127.0.0.1:<ephemeral-port>` listener. The task and script
returned 200. The constrained graph model returned the core projection and the
dynamically loaded `third.party.status` projection; it denied introspection and
mutation with 400. It is not a GraphQL service.

| Local threat attempt | HTTP result | Observation |
| --- | ---: | --- |
| no bearer to `/public/control` | 403 | local threat test: credential denied |
| wrong bearer | 403 | local threat test: credential denied |
| valid bearer plus foreign `Origin` | 403 | local threat test: origin denied |
| valid in-memory bearer | 405 | local threat test: no action exists |
| body larger than 1,024 bytes | 413 | local threat test: body denied |
| mobile endpoint without evidence | 404 | endpoint unregistered |

The summary records `mutations: 0`. The public results are local denial
observations, not public-front-door evidence. No CORS header is emitted.

Google Chrome and Firefox binaries were attempted. Firefox caused the required
semantic evidence: it POSTed `/graph`, parsed the exact response, and made the
validated browser-evidence POST observed by the server. Chrome made no semantic
loopback request. Their exit outcomes were deliberately not used. The retained
browser results are Firefox `PASS: POST /graph and response validated` and
Chrome `BLOCKED: no semantic loopback request`; the aggregate is `MIXED`, not
browser support or a cross-browser pass.

## Disposition limits and cleanup

`exec_module` dynamically ran the controlled module with the harness process's
authority. Returned-shape validation does not sandbox, sign, isolate, revoke,
or make that extension safe; the extension path is rejected. The model has no
variables, pagination, cost limits, nested graph, authentication, or standard
GraphQL semantics. Mobile remains blocked without reviewed unique evidence.

The browser, graph, extension, and public-control models remain local only. No
port was exposed beyond loopback and no deployment occurred. Re-run the three
commands above in a suitable browser environment before claiming cross-browser
semantic evidence; delete only the named owned roots after inspection.
