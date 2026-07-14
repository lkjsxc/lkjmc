# Inventory sync

## Purpose

This document owns local slot-8 token repair.

## Status

implemented

## Repair triggers

Paper repairs the token after join, respawn, pickup, and cancelled token
movement. It removes duplicate tokens outside slot `8` and restores the local
`NETHER_STAR` token there.

## Safety rules

Repair uses only current Bukkit inventory state. It reads no daemon setting,
credential, database, filesystem, network, or process. Opening the token selects
`root` and separately reads immutable cached A-JVM snapshots without waiting.

## Failure behavior

Repair never reports a daemon mutation result. Snapshot outage affects dependent
menu rows truthfully while root navigation and curated documentation remain
available.
