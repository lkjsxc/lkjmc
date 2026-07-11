# Current blockers

## Purpose

This document names active work and external proof gaps. Controller state remains
in `tmp/lkjmc-autonomous-evolution-plan/control/planctl.py`; this file does not
claim, complete, or transition a task.

## Completed foundation

Shipped behavior is bounded by the capability matrices in
[State](../state/README.md). Historical task records remain in
[Archive](archive/README.md); neither is an implementation plan or live proof.

## Active blockers

- The documentation gate is in its `D-STATE` capability-matrix slice. Its
  independent `D-VERIFY` review must examine the committed diff before the
  controller can decide the next transition.
- Nested Gradle output can make `check-lines.py` fail after a build although a
  clean-tree run passes. `F-SAFE-OPS` owns the guard repair.
- Guarded Minecraft, Bedrock, Discord, web, and Kubernetes lanes need their
  exact external prerequisites. A missing prerequisite is a skip or failure,
  never a pass.

## Live prerequisites

- Minecraft: `LKJMC_MINECRAFT_SMOKE=1`.
- Minecraft claim: `LKJMC_MINECRAFT_CLAIM_SMOKE=1`.
- Playable: `LKJMC_PLAYABLE_SMOKE=1` and `LKJMC_ACCEPT_MINECRAFT_EULA=1`.
- Bedrock: `LKJMC_BEDROCK_SMOKE=1` plus a supported endpoint and client.
- Discord: `LKJMC_DISCORD_SMOKE=1` plus real credentials and interaction access.
- Web: `LKJMC_WEB_SMOKE=1` plus daemon and browser prerequisites.
- Kubernetes: `LKJMC_KUBERNETES_SMOKE=1`, `kubectl`, and an authorized
  disposable namespace.

## Next executable step

Complete the `D-STATE` evidence and request an independent `D-VERIFY` review;
the controller alone records any transition.
