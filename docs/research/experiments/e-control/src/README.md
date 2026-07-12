# E-CONTROL harness source

## Purpose

These modules implement only the disposable comparison slice. They do not link
to the product workspace.

## Table of contents

No child documentation files exist in this source directory.

## Layout

`main.rs` owns safety-gated candidate execution and invariant reporting.
`lab.rs` owns disposable URL validation, database health, schema lifecycle, and
real cancellation. `async_exec.rs` implements concurrent keyed admission,
controlled saturation of its real ingress pool, and a post-launch journal
interruption routed through that pool. `actor.rs` implements bounded
key-sharded actors and controlled saturation of an actual actor mailbox.
`effects.rs` owns nonblocking bounded effect submission, controlled saturation
of its real effect pool, durable child claims, and interruption cleanup. The
`sync_*` modules retain the comparable synchronous baseline and bounded worker
pool.
