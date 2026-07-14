# Interaction contract

## Purpose

This contract defines inventory input and action dispatch.

## Status

implemented

## Metadata

Every actionable item carries route id, session id, request id, render revision,
slot, and action id. The listener cancels top-inventory clicks before decoding.
An empty slot, malformed metadata, route mismatch, stale session, stale render,
or unknown action is inert and returns localized chat fallback.

## Session behavior

Navigation and Back replace the inventory without an intermediate close. Only
`CLOSE` calls close. One asynchronous click is admitted per session. Repeated
clicks while pending are rejected. Open, navigation, and refresh advance a
per-player token containing adapter instance, generation, locale, route,
session, and request. Close, reopen, locale change, disconnect, adapter
replacement, and disable invalidate prior tokens.

A response may update only the matching current token. It is dropped silently
unless adapter, generation, locale, player, session, route, and request still
match immediately before application; stale work cannot open, close, overwrite,
or message.

## Authority

Read-only navigation needs no mutation grant. A mutation requires current typed
snapshots, exact capability, trusted attestation, and an implemented typed port.
Any missing condition denies without a command, request body, or success claim.

## Threading

Minecraft scheduler callbacks only validate, render, submit bounded work, or
apply Bukkit effects. Pending player responses return through player/entity
ownership on Folia; the global scheduler is only for global-safe work. These
callbacks never block on database, filesystem, network, process, download, or
worker completion.
