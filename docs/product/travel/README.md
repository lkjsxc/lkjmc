# Travel

## Purpose

This area owns player travel surfaces that are not server lifecycle transfer
flows.

## Status

implemented

Implemented: selected-home detail routes and paid dimension random-teleport profile
menus.

## Table of contents

- [Homes](homes.md)

## Contract

Travel actions must be real, scheduler-safe, and daemon-backed when durable
state is involved. Ordinary home teleport and free overworld random teleport do
not require confirmation. Destructive or paid dimension-changing travel does.

## Outcome, journey, and evidence boundary

A player selects a saved destination or obtains a fresh random-teleport quote,
then receives a scheduler-safe move or an exact disabled/failure reason. Unknown
or unjoinable cross-server homes do not transfer; paid profiles do not charge
when safe search fails and use their refund path on final teleport failure.
Store and adapter tests cover documented paths, while live cross-server movement
requires the guarded playable environment.
