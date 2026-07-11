# Network product contract

## Purpose

This directory owns user-visible contracts for the default playable network and
server lifecycle behavior.


## Status

implemented

## Table of contents

- [Bedrock entry](bedrock-entry.md)
- [Domain entry](domain-entry.md)
- [Java compatibility](java-compatibility.md)
- [Playable default](playable-default.md)
- [Random teleport and portals](random-teleport.md)
- [Server lifecycle](server-lifecycle.md)

## Contract

The Java network must be playable even when optional Bedrock or protocol
compatibility features are withdrawn with diagnostics. Lifecycle state must be
truthful and explain stopped, starting, running, suspended, and unavailable
servers.

## Outcome, journey, and evidence boundary

A Java player can enter through the configured proxy and select only a ready,
registered, joinable backend. Optional Bedrock and protocol compatibility can be
withdrawn with a diagnostic while Java play remains available; unready backends
stay visible with disabled reasons. Deterministic lifecycle checks support state
reduction; live reachability and optional protocol claims require their guarded
smokes and prerequisites.
