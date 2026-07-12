# Security commands

## Purpose

This generated file lists `security` daemon command literals from
[contracts/commands/README.json](../../../../../contracts/commands/README.json).

## Status

implemented

## Commands

| Command | Authorization | Surfaces | Summary |
| --- | --- | --- | --- |
| `security.daemon-token.create` | admin | cli | Create a bounded scoped daemon credential in an owner-limited file without returning its value. |
| `security.daemon-token.plan` | admin | cli | Execute security daemon-token plan behavior for the security command family. |
| `security.daemon-token.revoke` | admin | cli | revoke a scoped daemon HTTP token by credential id. |
| `security.daemon-token.rotate` | admin | cli, web | Execute security daemon-token rotate behavior for the security command family. |
| `security.daemon-token.status` | admin | cli, web | Execute security daemon-token status behavior for the security command family. |
| `security.daemon-token.verify` | admin | cli | Execute security daemon-token verify behavior for the security command family. |
