# End expedition

## Purpose

This contract defines the durable `end-expedition` temporary-adventure catalog
entry.

## Status

partial

Missing: attested Java authority to obtain player consent, purchase, transfer,
or report this adventure.

## Durable contract

A root-authorized daemon operation may validate the catalog entry, settle one
point spend, create an adventure session and temporary instance, and record
startup, refund, and cleanup facts. Start or readiness failure after settlement
uses the documented idempotent refund path. These facts do not prove a player
transfer, inventory delivery, or Java success message.

## EULA boundary

Adventure purchase and temporary-instance planning contain no Minecraft EULA
boolean and do not claim process startup. The systemd/Rust start boundary uses
the same root-owned host policy as every other managed server kind and
atomically verifies that instance's `eula.txt` before spawn. Missing policy or
unsafe materialized state keeps the instance stopped; player or request data
cannot create or upgrade host consent.

## Shop delivery contract

Only shop item `adventure-end-expedition` with metadata exactly
`{"delivery":{"executor":"adventure","adventureId":"end-expedition"}}`
can designate this entry. The store and database reject every retired executor,
custom adventure item, alternate adventure id, added metadata field, or
noncanonical delivery before settlement. The shop handler classifies stored
metadata, never caller-supplied metadata, and uses the fixed `end-expedition`
id with no fallback.

## Java boundary

Paper/Folia and Velocity register no End Expedition command, menu,
confirmation, transfer, return, profile handoff, or bridge. Java daemon adapters
remain withdrawn pending trusted identity/session attestation.
