# Dynamic menus

## Purpose

This document defines dynamic rendering from typed revisioned snapshots.

## Status

implemented

## State model

A route dependency is current, stale, or unavailable. Current snapshots render
only data understood by the route's closed binding. Stale snapshots retain
readable rows with a visible stale warning and disable mutations. Unavailable
snapshots render localized failure copy and no fabricated empty state.

Menu, permission, claim, and settings payloads use generated A-JVM records.
Routes that depend on profile, routing, or presence use those generated records.
A payload/domain/key mismatch fails the whole view.

## Refresh and correlation

Refresh starts one request for the session. Metadata records request and render
revision. A completion updates only the same open session and request; an old
completion is inert. A click from an older render is rejected before action
dispatch.

## Mutation boundary

Dynamic row mutations use closed operation identifiers. They require a current
capability and attestation. Absence is a localized denial, never a fake success
or hidden command.
