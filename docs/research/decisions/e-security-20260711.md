# E-SECURITY credential research decision

## Purpose

Preserve the evidence-backed disposition of credential candidates without
adopting any candidate into product authentication.

## Disposition

`combine`, not adopt. Evidence is the committed
[E-SECURITY run](../runs/e-security-20260711.md), hypothesis, initial probe
`5a567e1aa9cf44a7d459eaf8854c9267ac1cd44e`, and correction probe
`f0c975bccf7a06c5c1ea21a96b22a60cb86bacf4`.

## Findings

Direct PostgreSQL lookup enforced stored surface, principal, scope, expiry, and
revocation facts but incurred the induced 107,085 us median delay. The actual
synchronous-driver reactor attempt panicked, so direct lookup is rejected on an
async reactor path. The isolated worker path kept 22 ticker intervals runnable
during its delayed lookup.

The bounded cache returned 0 us median hits in this workload. A live PostgreSQL
revision listener received a normal notification. In the loss falsification, a
dropped listener was absent while revocation and notification occurred; the new
listener received no old message. The test candidate then denied reads while
disconnected and after reconnect until a revision lookup repaired it. The
revision changed to 2, the entry was invalidated, and a 50 ms missed-repair
deadline also denied. Its measured repair lookup took 1,209 us.

This is a test-only bounded fail-closed result, not evidence of a shipped
listener or periodic repair implementation. It does not establish production
reconnect timing, listener ownership, database-outage behavior, capacity, or a
safe repair cadence. Adoption must implement and independently falsify all of
those conditions; it must fail closed on cache, listener, repair, or database
uncertainty.

The signed 250-millisecond credential rejected alteration, wrong surface, early
clock, and expiry, but remained valid after the database credential was
revoked. Reject signed-only credentials. It could only be reconsidered with a
verified revocation check or a shorter independently justified bound.

Unix peer identity produced actual kernel peer credentials independently of
bearer credentials. A disposable Docker client ran as `--user=65534:65534`
against a read-only bind-mounted host socket. The listener observed UID 65534,
different from its UID-1001 socket owner, and denied the peer. This replaces the
blocked `setpriv` attempt; it is actual different-UID evidence, although it is
still a test harness rather than a daemon transport adoption. Session
expiry/logout and pre-lookup rate pressure passed in the isolated tests.

## Safety and next step

Keep F-SAFETY containment unchanged: do not route cache, signed, rate, or Unix
peer experiments into the daemon; do not relax existing authorization; and do
not render root tokens. The initial scoped-store creation failure is a separate
baseline defect, not permission to bypass it in product code.

No owner or state document changes are warranted because this branch ships no
behavior. The next executable step is an independent verifier running the
harness and its separate replay root, inspecting the redacted raw logs, and
repeating notification-loss and Docker peer falsification before any synthesis
discussion.
