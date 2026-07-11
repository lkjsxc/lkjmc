# Linking

## Purpose

This document defines Minecraft-to-Discord account linking.


## Status

partial

Missing: a trusted Discord interaction policy before Discord can complete or use
a link.

## Flow

A Minecraft player runs the link command, which calls `player.link.begin` and
receives a one-time code. The daemon stores only a SHA-256 hash of the code, with
a ten-minute expiry and one active unconsumed code per player. The plaintext code
is returned once to the player and must not be logged.

Discord completion and removal are withdrawn. The code remains unconsumed until
a future trusted Discord interaction policy is implemented; only Minecraft can
revoke an existing record through `player.link.remove`.

## Failure reasons

Invalid, expired, or reused codes fail with a typed daemon error when a trusted
completion path exists. Discord wake requests are withdrawn. Removed links must
not authorize future linked-player actions.

## Commands

`player.link.begin` and `player.link.remove` are the current durable command
contract. No Discord command is currently part of the flow.
