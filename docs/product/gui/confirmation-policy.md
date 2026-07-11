# Confirmation policy

## Purpose

This document records the Java confirmation-menu withdrawal boundary.

## Status

implemented

## Current boundary

Inventory confirmation routes for destructive state, purchases, moderation,
security, instances, and EULA acceptance are withdrawn pending trusted
identity/session attestation. Local documentation browsing has no mutation and
requires no confirmation.

## Daemon boundary

Daemon and CLI operations retain their own owner-defined confirmation and consent
rules. No Java menu, direct command, or shop action may originate EULA consent
or substitute a local success response.

## Verification

Daemon tests cover their owner rules. Java containment inspection proves no
confirmation route or mutation action is packaged.
