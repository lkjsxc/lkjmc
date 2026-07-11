# Server lifecycle

## Purpose

This contract defines daemon server create, start, readiness, and observation
states.

## Status

implemented

## Workflow

Server create and start persist durable intent, allocate a port, render files,
install required assets, start the runtime, wait for readiness, and report an
observation truthfully. A ready backend does not claim Velocity registration or
player joinability while the Java daemon adapter is withdrawn pending trusted
identity/session attestation.

## Create planning

`instance.create.plan` returns structured prerequisites, jar asset candidates,
plugin asset status, port plan, and runtime adapter. An EULA-gated plan with
absent or false consent returns `adventure.confirmation_required` before
database planning. CLI and web provide documented operator guidance; no Java
menu creates or starts an instance.

## Observation

Stopped, starting, failed, suspended, hidden, and ready state are durable daemon
facts. Connect-host derivation remains local-process, Compose, or Kubernetes
runtime configuration. It is not a Java proxy registration result.

## Verification

Core, daemon, and store tests cover lifecycle state and observations. Java
containment inspection proves no Velocity registration report or transfer path
is packaged.
