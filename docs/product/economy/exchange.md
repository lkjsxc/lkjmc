# Item exchange

## Purpose

This document defines durable item-to-points exchange records and the Java
withdrawal boundary.

## Status

implemented

## Current behavior

The daemon owns exchange rates and idempotent exchange commits. Paper
`/exchange`, inventory counting/removal, reconciliation, refund, and playable
smoke behavior are withdrawn pending trusted identity/session attestation.

## Required behavior

`COBBLESTONE` has a configured one-point-per-block rate. Additional rates are
configurable and must not create buy/sell profit loops with shop prices.

## Daemon boundary

A daemon caller supplies a material, amount, player identity, and correlation.
The handler validates the rate and records ledger and exchange event atomically.
A replay returns the settled event without a second grant. No Java adapter may
turn this record into inventory mutation or a player success message.

## Diagnostics

Invalid material, disabled rate, duplicate correlation, database failure, and
contained ambiguity are typed daemon states. They must not expose credentials,
database URLs, raw JSON, or stack traces.
