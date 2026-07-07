# Security commands

## Purpose

This generated file lists `security` daemon command literals from
[contracts/commands.json](../../../../../contracts/commands.json).

## Status

implemented

## Commands

| Command | Authorization | Surfaces | Summary |
| --- | --- | --- | --- |
| `security.daemon-token.create` | admin | cli, paper, velocity, web | create a scoped daemon HTTP token and return the raw value once. |
| `security.daemon-token.plan` | admin | cli, discord, paper, velocity, web | Execute security daemon-token plan behavior for the security command family. |
| `security.daemon-token.revoke` | admin | cli, paper, velocity, web | revoke a scoped daemon HTTP token by credential id. |
| `security.daemon-token.rotate` | admin | cli, discord, paper, velocity, web | Execute security daemon-token rotate behavior for the security command family. |
| `security.daemon-token.status` | admin | cli, discord, paper, velocity, web | Execute security daemon-token status behavior for the security command family. |
| `security.daemon-token.verify` | admin | cli, discord, paper, velocity, web | Execute security daemon-token verify behavior for the security command family. |
