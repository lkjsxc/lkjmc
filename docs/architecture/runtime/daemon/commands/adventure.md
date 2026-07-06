# Adventure commands

## Purpose

This generated file lists `adventure` daemon command literals from
[contracts/commands.json](../../../../../contracts/commands.json).

## Status

implemented

## Commands

| Command | Authorization | Surfaces | Summary |
| --- | --- | --- | --- |
| `adventure.catalog.list` | open | cli, discord, paper, velocity, web | adventure_api.rs; returns enabled catalog. |
| `adventure.end.purchase` | open | cli, discord, paper, velocity, web | adventure_api.rs; compatibility path delegating. |
| `adventure.end.return` | open | cli, discord, paper, velocity, web | adventure_api.rs; compatibility path delegating to. |
| `adventure.purchase` | open | cli, discord, paper, velocity, web | adventure_api.rs; purchases any enabled catalog. |
| `adventure.return` | open | cli, discord, paper, velocity, web | adventure_api.rs; validates an active adventure. |
| `adventure.session.cancel` | admin | cli, discord, paper, velocity, web | adventure_api.rs; admin cancellation for a. |
| `adventure.session.get` | open | cli, discord, paper, velocity, web | adventure_api.rs; returns a player's active. |
| `adventure.session.list` | admin | cli, discord, paper, velocity, web | adventure_api.rs; admin status list for recent. |
