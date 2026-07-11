# Transfer safety

## Purpose

This document records the withdrawal boundary for Java profile transfer.

## Status

implemented

## Current boundary

Daemon `player.transfer.saved` and `player.recovery.report` handlers retain
audit-backed transfer acknowledgement and recovery data. Paper/Folia snapshot
save/load, plugin messages, Velocity transfer, cross-server teleport, and menu
transfer actions are withdrawn pending trusted identity/session attestation.
A durable transfer intent is not proof that a player moved.

## Future rule

A future transfer adapter must obtain trusted authenticated player identity and
session attestation, save a leased snapshot off the scheduler, acknowledge the
exact revision, and wait for an actual connection result. It must deny uncertain
saves rather than claim a target arrival.

## Evidence boundary

Store tests prove record and recovery semantics only. Java containment inspection
proves no transfer bridge or daemon client is packaged.
