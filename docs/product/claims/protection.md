# Claim protection

## Purpose

This document records the withdrawal boundary for Java claim protection.

## Status

implemented

## Current boundary

Claim records and pure policy types remain daemon/store data. Paper/Folia claim
refresh, block-event listeners, `/claim`, and live protection decisions are
withdrawn pending trusted identity/session attestation. No Java plugin queries
or caches claim state.

## Future rule

A future listener must obtain trusted authenticated player identity and session
attestation before it can use a claim decision. It must refresh off Minecraft
scheduler threads and never infer authority from a request body, `op`, cached
grant, or token file.

## Verification

Store and pure-policy tests prove data and decisions only. Java containment
inspection proves no claim daemon client, refresh service, listener, or command
is packaged.
