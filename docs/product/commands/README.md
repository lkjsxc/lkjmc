# Commands

## Purpose

This area owns Minecraft and SSH CLI command contracts.


## Status

implemented

## Table of contents

- [Minecraft commands](minecraft.md)
- [Completion](completion.md)
- [SSH CLI](ssh-cli.md)

## Rule

Commands parse, authorize, and delegate. Business logic belongs in core planners
or daemon handlers. Completion uses cached context and must never block a
Minecraft scheduler thread on daemon, database, filesystem, or network work.

## Outcome, journey, and evidence boundary

A player or operator enters a documented command, sees permission-filtered
completion where available, and receives product output, usage, denial, or a
safe typed diagnostic. Cached completion may omit dynamic candidates when the
daemon is unavailable; execution authorization remains daemon-final. Parser and
adapter tests prove registered contract paths, while live command execution is
proved only by the guarded playable smoke when its prerequisites are supplied.
