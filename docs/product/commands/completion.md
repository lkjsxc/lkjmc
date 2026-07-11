# `/lkjmc` completion

## Purpose

This document records the withdrawal of `/lkjmc` completion from Paper/Folia
and Velocity.


## Status

implemented

## Contract

No Paper/Folia or Velocity `/lkjmc` command or completion is registered. The
shared command tree and its daemon-capable adapters are withdrawn pending
trusted identity/session attestation.

## Inputs

Paper retains only local-safe `/menu` and `/docs` entrypoints plus hotbar/docs
UI. Velocity retains only MOTD and tab-list behavior. Neither surface obtains
completion context, grants, or daemon data.

## Edge cases

A future reintroduction needs a separate owner contract, trusted authenticated
player identity and session attestation, registration proof, and artifact
inspection. It must not treat cached grants or caller-shaped actor fields as
identity proof.

## Nonblocking rule

There is no Java daemon completion refresh or grant snapshot cache in the
shipped plugins.
