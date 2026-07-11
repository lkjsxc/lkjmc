# Announcements

## Purpose

This document owns authorized network announcements and their player-facing
outcome.

## Status

implemented

## Current journey

An authorized actor supplies a server target and message. The daemon records the
announcement; a supported adapter broadcasts localized presentation copy to the
targeted player surface. Denial, malformed input, or daemon failure returns a
safe failure and never presents an announcement as sent.

## Surface states

Loading occurs while the adapter requests the daemon result. There is no player
empty state for a single send. A transient failure may be retried by the actor;
there is no automatic replay because it could duplicate a broadcast. Recovery is
the durable record and audit result, not inferred player delivery.

## Boundaries

Current source supports daemon creation and recent reads. Paper and Discord
command adapters are withdrawn; no Java plugin broadcasts from a daemon record.
This does not claim delivery to disconnected players, another server, or an
external channel.
