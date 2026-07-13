# Adventure lifecycle

## Purpose

This document owns the durable temporary-adventure lifecycle boundary.

## Status

partial

Missing: attested Java adapter authority for purchase, transfer, return, and
player feedback.

## Durable behavior

Root-authorized daemon operations may atomically validate a catalog entry,
record one point debit, create one fenced adventure lifecycle, and append its
change-feed fact. Legal revisions move through pending/start intent and then
wait for trusted runtime observation. Failure may atomically record one refund;
cleanup remains pending until a separately authenticated observation. Exact
correlation replay is stable, stale fences/revisions and competing terminal
outcomes are denied.

Durable debit, refund, startup intent, cleanup intent, and failure facts do not
imply a runtime effect, player transfer, inventory receipt, or cleanup. Without
future trusted acknowledgements, effect edges remain pending or failed.

## Withdrawn adapter behavior

Paper/Folia and Velocity do not request an adventure, start a backend for a
player, issue a transfer intent, connect or return a participant, or report an
adventure result. A request body party, EULA flag, player id, or target server is
not adapter authentication or authorization evidence.
