# Travel

## Purpose

This area owns player travel surfaces that are not server lifecycle transfer
flows.

## Status

partial

Missing: selected-home detail routes and paid dimension random-teleport profile
menus.

## Table of contents

- [Homes](homes.md)

## Contract

Travel actions must be real, scheduler-safe, and daemon-backed when durable
state is involved. Ordinary home teleport and free overworld random teleport do
not require confirmation. Destructive or paid dimension-changing travel does.
