# Commands

## Purpose

This area owns Minecraft and SSH CLI command contracts.

## Table of contents

- [Minecraft commands](minecraft.md)
- [Completion](completion.md)
- [SSH CLI](ssh-cli.md)

## Rule

Commands parse, authorize, and delegate. Business logic belongs in core planners
or daemon handlers. Completion uses cached context and must never block a
Minecraft scheduler thread on daemon, database, filesystem, or network work.
