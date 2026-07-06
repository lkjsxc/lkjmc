# Economy menu routes

## Purpose

This generated file lists `economy` menu routes from
[contracts/menus](../../../../contracts/menus).

## Status

implemented

## Routes

| Route | Kind | Parent | Binding | Source | Data commands | Confirmation |
| --- | --- | --- | --- | --- | --- | --- |
| [`daily`](../../../../contracts/menus/daily.json) | list | economy | daily | daemon | `player.daily.status` | — |
| [`economy`](../../../../contracts/menus/economy.json) | static | root | — | — | — | — |
| [`kits`](../../../../contracts/menus/kits.json) | list | economy | kits | daemon | `player.kit.list` | — |
| [`shop`](../../../../contracts/menus/shop.json) | list | economy | shop | daemon | `player.shop.list`, `player.points.balance` | — |
| [`votes`](../../../../contracts/menus/votes.json) | list | economy | votes | daemon | `player.vote.list` | — |
