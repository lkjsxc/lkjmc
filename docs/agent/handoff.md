# Handoff

## Purpose

This document defines the truthful final response format for agents.

## Required sections

- Summary: completed task slice and commit.
- Docs: paths changed, or `none`.
- Implementation: paths changed, or `none`.
- Verification: each exact command and whether it passed, failed, or skipped.
- Not tested: available but unrun checks and why.
- Risks: known defect, assumption, or integration concern, or `none`.
- Next executable step: one concrete command or task for the next owner.

## Evidence rule

Never claim a command passed unless it ran in the current work session. Include
a compact result summary, worktree commit, and any source evidence needed to
interpret the change. Do not present a skipped guard, clean diff, or unrun gate
as a pass. Do not claim controller transition; report the evidence for the
controller to resume from instead.
