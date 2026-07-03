# Linking

## Purpose

This document defines Minecraft-to-Discord account linking.


## Status

implemented

## Flow

A Minecraft player runs the link command, which calls `player.link.begin` and
receives a one-time code. The daemon stores only a SHA-256 hash of the code, with
a ten-minute expiry and one active unconsumed code per player. The plaintext code
is returned once to the player and must not be logged.

The Discord user runs `/lkjmc link code:<code>`, which delegates
`discord.link.complete`. A valid unexpired code creates or replaces the durable
Discord account link and consumes the code. `player.link.remove` and
`discord.link.remove` revoke the link from either side.

## Failure reasons

Invalid, expired, or reused codes fail with a typed daemon error. Linked Discord
wake requests resolve the verified Minecraft UUID and delegate the normal
wake-and-join queue. Unlinked users receive a typed daemon error. Removed links
must not authorize future linked-player actions.

## Commands

`player.link.begin`, `player.link.remove`, `discord.link.complete`,
`discord.link.remove`, and `discord.wake.request` are the durable command
contract for this flow.
