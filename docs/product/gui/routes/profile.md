# Profile menu routes

## Purpose

This generated file lists `profile` menu routes from
[contracts/menus](../../../../contracts/menus).

## Status

implemented

## Routes

| Route | Kind | Parent | Binding | Source | Data commands | Confirmation |
| --- | --- | --- | --- | --- | --- | --- |
| [`achievement-detail`](../../../../contracts/menus/achievement-detail.json) | detail | achievement-directory | achievement-detail | daemon | `player.achievements.list` | — |
| [`achievement-directory`](../../../../contracts/menus/achievement-directory.json) | list | achievements | achievement-directory | daemon | `player.achievements.list` | — |
| [`achievements`](../../../../contracts/menus/achievements.json) | list | profile | achievements | daemon | `player.achievements.list` | — |
| [`profile`](../../../../contracts/menus/profile.json) | detail | root | profile | daemon | `player.points.balance`, `player.achievements.list` | — |
