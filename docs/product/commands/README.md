# Commands

## Purpose

This area owns local Minecraft documentation entrypoints and SSH CLI command
contracts.

## Status

implemented

## Table of contents

- [Minecraft commands](minecraft.md)
- [Completion](completion.md)
- [SSH CLI](ssh-cli.md)

## Rule

CLI commands parse, authorize, and delegate to daemon handlers. Paper/Folia
register only local `/menu` and `/docs`; Velocity registers no command. Java
daemon completion and cached grant context are withdrawn pending trusted
identity/session attestation.

## Evidence boundary

CLI parsing and daemon tests prove their owner paths. Java containment inspection
proves local registration and jar absence; it does not prove a live player
session.
