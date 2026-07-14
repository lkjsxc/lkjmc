# Minecraft commands

## Purpose

This document defines the shipped in-game command surface and containment
boundary.

## Status

implemented

## Paper/Folia commands

- `/menu` opens the document-driven `root` route and requires
  `lkjmc.user.menu`.
- `/docs [search <query>|path]` opens a curated documentation route in the same
  engine and requires `lkjmc.user.docs`.

The slot-8 token opens `root`. These entrypoints render source-owned routes and
revisioned typed snapshot views. They do not register `/lkjmc`, a generic daemon
command, or a generic action body.

## Action boundary

Navigation and docs are local inventory effects. Stale and unavailable snapshot
states are labelled. Mutation actions require a current capability and trusted
attestation; the shipped runtime has no daemon mutation port and therefore
denies rather than dispatching or claiming success.

## Velocity behavior

Velocity provides MOTD and tab-list presentation only. It registers no `/lkjmc`,
`/hub`, transfer, or menu command.

## Verification

`menuProbes` drives production protocol adapter code without claiming a live
Minecraft client. JVM containment inspects source and real jars for one menu
engine, the compiled bundle, and absent withdrawn command/daemon surfaces.
