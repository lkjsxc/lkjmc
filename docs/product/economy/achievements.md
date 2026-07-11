# Achievements

## Purpose

This document owns durable achievement definitions, progress, and rewards.

## Status

implemented

## Current boundary

Definitions include id, category path, title key, description key, icon material,
criteria, threshold, hidden/repeatable flags, and reward entries. Daemon/store
operations can record progress and explicit idempotent claims.

Achievement directories, detail pages, claim buttons, Main Menu navigation, and
Java reward delivery are withdrawn pending trusted identity/session attestation.
A claim record is not a player-visible reward delivery.

## Verification

Daemon and store tests cover definition, progress, and claim records. Java
containment inspection proves no achievement menu or command adapter is packaged.
