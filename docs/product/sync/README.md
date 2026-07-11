# Player sync

## Purpose

This area owns durable profile data and the withdrawn Java synchronization
boundary.

## Status

implemented

## Table of contents

- [Player profile](player-profile.md)
- [Transfer safety](transfer-safety.md)

## Contract

The daemon and store own profile snapshots, leases, recovery records, and CLI
operations. Paper/Folia profile synchronization and Velocity transfer handling
are withdrawn pending trusted identity/session attestation. Process-only servers
remain managed but do not claim player sync.

## Evidence boundary

Store tests support durable snapshot behavior. They do not prove a Java save,
load, session, or transfer path.
