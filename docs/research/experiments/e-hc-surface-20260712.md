# E-HC-SURFACE bounded surface hypothesis

## Purpose

Test five costly surface candidates locally without registering a product route,
controller, extension point, public listener, or mobile API.

## ID and catalog evidence

`E-HC-SURFACE` covers `HC-BROWSER-SPA`, `HC-GRAPHQL`, `HC-PLUGIN-SDK`,
`HC-PUBLIC-CONTROL`, and `HC-MOBILE-CLIENT` from the
[high-cost catalog](../ideas/high-cost.md). The baseline is the shipped,
loopback-only authenticated `/web` presentation adapter and its daemon-command
boundary; this experiment does not call it or its controller.

## Hypothesis and real slice

A standard-library local harness will start one `127.0.0.1:0` server. It will
serve one separately named operator-task page, a constrained graph model, and a
deliberately disabled public-control path. The model is not a GraphQL service;
it resolves a fixed read-only projection from a dynamically loaded third-party
domain module, not a hard-coded extension response. A mobile-neutral endpoint
is registered only after separate evidence proves a non-duplicative need.

## Invariants

The harness may bind only loopback, use an ephemeral port, write only to an
owned `/tmp/lkjmc-e-hc-surface-*` root, and retain no credential in artifacts.
Its CSP permits `connect-src` only to that exact loopback listener. It must not
invoke product code, PostgreSQL, a controller, process management, or a public
deployment. Its public-control path may never mutate state, even with the
in-memory test bearer. Model mutation, introspection, unknown selections,
cross-origin control, bad bearer input, and oversize bodies must fail closed.
`exec_module` loading is unsandboxed host-authority execution and is rejected;
shape validation does not make it a plugin sandbox.

## Variants and combinations

Exercise the browser-task HTML over real loopback HTTP and query both the core
instance projection and loaded extension domain. Attempt model introspection
and mutation. Attempt public control with no bearer, a wrong bearer, a foreign
`Origin`, a valid bearer, and an oversize body. Attempt the mobile-evidence gate
with no file and a nonexistent file; neither may start a mobile endpoint.

The browser semantic probe attempts installed Google Chrome and Firefox against
the local task. The server accepts browser evidence only after it observes that
browser POST `/graph` and reports the exact parsed response. A process exit does
not count. An unavailable binary or missing semantic request is that browser's
`BLOCKED`, not browser support; a pass plus a block aggregates to `MIXED`. HTTP
assertions remain local transport evidence only.

## Workload and measurements

Use the public seed label `e-hc-surface-20260712`; one deterministic request per
case is sufficient to falsify the narrow protocol invariants, not to estimate
performance. Record selected query fields, loaded domain, response status,
state-mutation count, artifact secret scan, listener address class, browser
statuses and aggregate, and mobile gate result. Aggregate browser status is
`PASS` only if all attempts pass, `BLOCKED` only if all block, otherwise
`MIXED`. Keep raw summary JSON only in the owned root and omit bearer material.

## Harness

The committed local-only source is [the E-HC-SURFACE harness](e-hc-surface/README.md).
It has no imports from product code and is the exact source exercised by the run.

## Base, worktree, and allowed writes

Base: `4b9357a8e1a7949e0ebfe59c16af5196554f46cc`; worktree:
`/tmp/pi-agent-d02d42c0-0146-467-a9a8bcd9`. Allowed writes are
`docs/research/**` only. Owner evidence read before this hypothesis:
`docs/architecture/web/{README,routes,security}.md`,
`docs/architecture/runtime/daemon/transport.md`, and
`docs/operations/web-control.md`.

## External prerequisite

No external service is permitted. Browser proof requires a locally installed
headless browser capable of loading loopback. The harness preserves each binary
attempt; only all-blocked attempts aggregate as `browser=BLOCKED`. A future mobile
candidate requires a supplied JSON evidence file containing a unique capability
not provided by the graph model; its exact rerun command is recorded with the
run. Neither missing prerequisite is substituted with a fake client or API.
