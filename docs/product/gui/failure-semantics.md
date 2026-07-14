# Failure semantics

## Purpose

This document defines closed player-visible menu failures.

## Status

implemented

## Classes

The engine distinguishes malformed bundle, unknown route, missing parameter,
stale render, stale response, busy session, unavailable dependency, stale
dependency, permission denial, missing attestation, and unsupported typed
operation. Raw exceptions, JSON, credentials, paths, and transport bodies are
never player copy.

Current data may render active read rows. Stale data is visibly labelled and has
no mutation action. Unavailable data names the unavailable state and does not
look like an empty successful result. Daemon outage leaves local navigation and
curated docs usable while daemon-dependent routes show unavailable.

## Close and fallback

Only explicit Close closes an inventory. Every other failure preserves the open
view or replaces it with a failure view. If inventory rendering cannot safely
apply, localized chat is the fallback. Unknown or old metadata is inert.

## Locale

English and Japanese have identical keys and placeholder sets. Player-facing
labels also have non-color text; color tags cannot be the only signal.
