# Adventure lifecycle

## Purpose

This document owns the durable temporary-adventure lifecycle boundary.

## Status

partial

Missing: attested Java adapter authority for purchase, transfer, return, and
player feedback.

## Durable behavior

Root-authorized daemon operations may validate a catalog entry, record point and
session facts, allocate a temporary instance, apply cleanup policy, and record a
refund or failure. These operations remain separately audited and do not imply a
player transfer.

## Withdrawn adapter behavior

Paper/Folia and Velocity do not request an adventure, start a backend for a
player, issue a transfer intent, connect or return a participant, or report an
adventure result. A request body party, EULA flag, player id, or target server is
not adapter authentication or authorization evidence.
