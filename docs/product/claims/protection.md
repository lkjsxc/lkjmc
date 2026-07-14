# Claim protection

## Purpose

This document records the withdrawal boundary for Java claim protection.

## Status

implemented

## Current boundary

Claim records and pure policy types remain daemon/store data. Java common may
cache a revisioned per-instance claim snapshot, but Paper/Folia block-event
listeners, `/claim`, and live protection decisions remain withdrawn pending
trusted identity/session attestation. Unavailable or expired claim views are not
a permission to mutate.

## Future rule

A future listener must obtain trusted authenticated player identity and session
attestation before using a claim decision. It must read a fresh immutable view,
refresh off scheduler threads, and never infer authority from a request body,
`op`, cached grant, or token.

## Verification

PostgreSQL/HTTP/JVM tests prove snapshot revision and cache repair only. Java
containment proves no claim listener, command, mutation, or enforcement bridge is
packaged.
