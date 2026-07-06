# Instance commands

## Purpose

This generated file lists `instance` daemon command literals from
[contracts/commands.json](../../../../../contracts/commands.json).

## Status

implemented

## Commands

| Command | Authorization | Surfaces | Summary |
| --- | --- | --- | --- |
| `instance.create` | admin | cli, discord, paper, velocity, web | instance_lifecycle.rs; product surfaces must validate a. |
| `instance.create.plan` | open | cli, discord, paper, velocity, web | instance_create.rs; returns startable-create. |
| `instance.delete` | admin | cli, discord, paper, velocity, web | instance_lifecycle.rs. |
| `instance.heartbeat` | open | cli, discord, paper, velocity, web | instance_heartbeat.rs. |
| `instance.list` | admin | cli, discord, paper, velocity, web | instance_read.rs. |
| `instance.logs` | open | cli, discord, paper, velocity, web | instance_read.rs. |
| `instance.restart` | admin | cli, discord, paper, velocity, web | instance_lifecycle.rs. |
| `instance.start` | admin | cli, discord, paper, velocity, web | instance_lifecycle.rs. |
| `instance.stop` | admin | cli, discord, paper, velocity, web | instance_lifecycle.rs. |
| `instance.wake.cancel` | open | cli, discord, paper, velocity, web | instance_wake_join.rs; cancels the player's live row. |
| `instance.wake.cleanup` | admin | cli, discord, paper, velocity, web | instance_wake_join.rs; expires stale live rows. |
| `instance.wake.consume` | open | cli, discord, paper, velocity, web | instance_wake_join.rs; marks a ready row transferred. |
| `instance.wake.request` | open | cli, discord, paper, velocity, web | instance_wake_join.rs; queues a player for a. |
| `instance.wake.status` | open | cli, discord, paper, velocity, web | instance_wake_join.rs; returns durable wake state. |
