# E-HC-SURFACE local harness

## Purpose

This directory contains the disposable loopback model used by the E-HC-SURFACE
research run. It is neither a product component nor an extension SDK.

## Table of contents

- [Source](#source)
- [Run and containment](#run-and-containment)
- [Limits](#limits)

## Source

`harness.py` starts an ephemeral `127.0.0.1` server, dynamically executes a
controlled third-party Python module, and queries its read-only projection
through a constrained graph model. It is not a GraphQL service. The separate
`/operator-task` page uses an exact-loopback CSP `connect-src` and a same-origin
script. The server accepts browser evidence only after Chrome or Firefox POSTs
`/graph` and reports the exact parsed response; process exit is ignored. The
summary preserves each browser's `PASS` or `BLOCKED` status and aggregates to
`PASS` only when all pass, `BLOCKED` only when all block, otherwise `MIXED`.

`/public/control` exists solely as a local denial threat test. It has no action
handler and returns 405 after a valid in-memory test bearer; every other threat
case denies earlier. The token is never printed, rendered, or persisted. The
root retains only `.owned` and a sanitized JSON summary. The recursive scan
covers every retained file and asserts no token, `__pycache__`, or temporary
`third_party_status.py` import artifact remains.

## Run and containment

```sh
python3 docs/research/experiments/e-hc-surface/harness.py --self-test \
  --artifact-root /tmp/lkjmc-e-hc-surface-main
python3 docs/research/experiments/e-hc-surface/harness.py --browser-semantic-test \
  --artifact-root /tmp/lkjmc-e-hc-surface-browser
python3 docs/research/experiments/e-hc-surface/harness.py --self-test \
  --mobile-evidence /tmp/lkjmc-e-hc-surface-missing.json
```

The first command owns a direct child of `/tmp` whose basename begins
`lkjmc-e-hc-surface-`; any other requested root is refused. The third command
must exit 2 with `mobile=BLOCKED` before creating a listener when the evidence
file is absent or invalid. A valid file needs `evidenceId` and a
`uniqueCapability` other than the existing graph projection; none is supplied
by this experiment.

Remove inspected evidence only after checking the path:

```sh
rm -rf -- /tmp/lkjmc-e-hc-surface-main
```

## Limits

The constrained model is not a GraphQL implementation. `exec_module` executes
an extension with the harness process's authority and is explicitly rejected:
returned-shape validation is not sandboxing, signing, isolation, or revocation.
A per-browser `BLOCKED` result means that browser had no semantic POST/response
proof; `MIXED` preserves a passing browser without claiming cross-browser
support. A pass is still not human workflow, TLS/front-door safety, public
exposure, mobile need, throughput, or a safe product integration.
