# Bedrock entry

## Purpose

This contract defines optional Bedrock support for the playable network.


## Status

implemented

## Default policy

Bedrock mode defaults to auto. In auto mode, Java play remains the product goal.
Bedrock is enabled only when Geyser and Floodgate assets are hash verified,
configuration is safe, and UDP `19132` can be bound or published.

## Network

Bedrock uses UDP, commonly `0.0.0.0:19132`. No current packaged deployment or
Compose service publishes that port; exposure requires a separate current product decision and evidence.

## Plugin placement

Geyser and Floodgate install on the Velocity proxy by default. Floodgate is
installed on backends only when backend API access is explicitly requested.

## Diagnostics

If UDP is unavailable, downloads cannot be verified, or Floodgate key handling is
unsafe, bootstrap withdraws Bedrock in auto mode and explains the reason. If the
operator explicitly enables Bedrock, the same condition may block bootstrap.
