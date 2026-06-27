# Network product contract

## Purpose

This directory owns user-visible contracts for the default playable network and
server lifecycle behavior.

## Table of contents

- [Bedrock entry](bedrock-entry.md)
- [Domain entry](domain-entry.md)
- [Java compatibility](java-compatibility.md)
- [Playable default](playable-default.md)
- [Server lifecycle](server-lifecycle.md)

## Contract

The Java network must be playable even when optional Bedrock or protocol
compatibility features are withdrawn with diagnostics. Lifecycle state must be
truthful and explain stopped, starting, running, suspended, and unavailable
servers.
