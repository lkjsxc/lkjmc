# Menu data contracts

## Purpose

This file defines the shipped local documentation-menu data contract.

## Status

implemented

## Bundled data

The Paper plugin consumes one generated documentation bundle. Its local helpers
provide normalized paths, titles, wrapped document lines, search matches, and
pagination. The menu never reads a daemon response, database row, token file,
or host filesystem path.

## Local effects

A local item may open a bundled path, change a document page, return to the
document list, start local search, or close the inventory. Missing or malformed
local metadata is inert or returns to local search. It never produces an
inventory delivery, player mutation, transfer, or daemon request.

## Withdrawn data

Server, admin, homes, warps, teleports, shop, adventures, achievements,
settings, claims, profile, and every other daemon-backed route family are not
packaged in a Java plugin. Their data bindings, grant checks, caches, and
mutation metadata remain withdrawn pending trusted identity/session attestation.

## Verification

Bundle and JVM containment checks cover only this local data contract.
