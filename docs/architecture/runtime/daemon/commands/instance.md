# Instance commands

## Purpose

This generated file lists `instance` daemon command literals from
[contracts/commands/README.json](../../../../../contracts/commands/README.json).

## Status

implemented

## Commands

| Command | Authorization | Surfaces | Summary |
| --- | --- | --- | --- |
| `instance.create` | admin | cli | Execute instance create behavior for the instance command family. |
| `instance.create.plan` | operator | internal | Execute instance create plan behavior for the instance command family. |
| `instance.delete` | admin | cli | Execute instance delete behavior for the instance command family. |
| `instance.list` | admin | cli, web | Execute instance list behavior for the instance command family. |
| `instance.logs` | operator | cli | Execute instance logs behavior for the instance command family. |
| `instance.restart` | admin | cli, web | Execute instance restart behavior for the instance command family. |
| `instance.start` | admin | cli, web | Execute instance start behavior for the instance command family. |
| `instance.stop` | admin | cli, web | Execute instance stop behavior for the instance command family. |
| `instance.wake.cancel` | operator | internal | Execute instance wake cancel behavior for the instance command family. |
| `instance.wake.cleanup` | admin | internal | Execute instance wake cleanup behavior for the instance command family. |
| `instance.wake.consume` | operator | internal | Execute instance wake consume behavior for the instance command family. |
| `instance.wake.request` | operator | internal | Execute instance wake request behavior for the instance command family. |
| `instance.wake.status` | operator | internal | Execute instance wake status behavior for the instance command family. |
