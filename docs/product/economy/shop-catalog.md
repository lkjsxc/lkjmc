# Shop catalog

## Purpose

This document defines daemon-owned point shop catalog and settlement data.

## Status

implemented

## Current boundary

Bootstrap, CLI, and root-authorized daemon operations seed and maintain catalog
data. Paper shop menus, balance lore, inventory delivery, refunds, adventure
confirmation, and transfer reporting are withdrawn pending trusted
identity/session attestation.

## Canonical adventure item

The only adventure delivery is item `adventure-end-expedition` with metadata
exactly `{"delivery":{"executor":"adventure","adventureId":"end-expedition"}}`.
Migration `042-canonical-adventure-shop-delivery.sql` normalizes only its known
canonical historical metadata forms. It preserves `shop_purchases`, stops with
an actionable diagnostic for any custom adventure or retired-executor row, and
installs a constraint rejecting every other adventure or retired executor.

Store upsert validates the same rule before SQL. The daemon classifies stored
item metadata, not caller metadata; it rejects noncanonical or retired delivery
and always uses fixed `end-expedition`, never a metadata fallback.

## Settlement and intent

A replay returns recorded purchase and pending-delivery facts without a second
debit. Settlement and durable delivery intent commit together, so debited value
is always accounted by one correlation. Unsupported executors, invalid
materials, disabled items, insufficient points, and noncanonical metadata fail
before deduction. Shop settlement carries no EULA confirmation. Any later
Minecraft process start independently requires the validated root-owned host
policy and exact per-instance materialized file.

The data owner cannot mark inventory receipt. Without a future authenticated
session/revision/fence-bound acknowledgement, delivery remains
`pending_receipt` or is explicitly `failed`; neither is player receipt. A daemon
settlement never claims an inventory item, player transfer, or Java success
message.

## Verification

Store tests cover catalog validation and migration behavior against PostgreSQL.
JVM containment inspection proves no shop menu, delivery, or refund adapter is
packaged.
