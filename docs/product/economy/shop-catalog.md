# Shop catalog

## Purpose

This document defines daemon-owned point shop catalog and settlement data.

## Status

implemented

## Current boundary

Bootstrap, CLI, and daemon administration can seed and maintain catalog data.
Daemon validation checks material and amount before settlement and writes
immutable purchase facts. Paper shop menus, balance lore, item delivery, refunds,
adventure confirmation, and transfer reporting are withdrawn pending trusted
identity/session attestation.

## Settlement rule

A replay returns the recorded purchase facts without a second debit. Unsupported
executors, invalid materials, amounts, disabled items, insufficient points, and
missing dependencies fail before deduction. A daemon settlement never claims an
inventory item, player transfer, or Java success message occurred.

## Consent boundary

The daemon keeps the documented adventure-consent classification before database
access. With Java confirmation menus withdrawn, no Java surface may originate
consent or invoke adventure delivery.

## Verification

Core and PostgreSQL-gated store tests cover price safety, replay, and invalid
metadata. Java containment inspection proves no shop menu, delivery, or refund
adapter is packaged.
