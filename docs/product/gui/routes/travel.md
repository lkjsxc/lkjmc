# Travel menu routes

## Purpose

This generated file lists `travel` menu routes from
[contracts/menus](../../../../contracts/menus).

## Status

implemented

## Routes

| Route | Kind | Parent | Binding | Source | Data commands | Confirmation |
| --- | --- | --- | --- | --- | --- | --- |
| [`home-create-confirm`](../../../../contracts/menus/home-create-confirm.json) | confirm | home-create-name | — | — | — | writes-named-durable-state |
| [`home-create-name`](../../../../contracts/menus/home-create-name.json) | custom | homes | — | — | — | — |
| [`home-detail`](../../../../contracts/menus/home-detail.json) | detail | homes | home-detail | daemon | `player.home.get` | — |
| [`homes`](../../../../contracts/menus/homes.json) | list | travel | homes | daemon | `player.home.list` | — |
| [`random-teleport-overworld`](../../../../contracts/menus/random-teleport-overworld.json) | detail | travel | random-teleport | daemon | `player.random-teleport.quote` | — |
| [`teleport-picker`](../../../../contracts/menus/teleport-picker.json) | list | teleports | teleport-picker | local | — | — |
| [`teleports`](../../../../contracts/menus/teleports.json) | list | travel | teleports | daemon | `player.snapshot` | — |
| [`travel`](../../../../contracts/menus/travel.json) | static | root | — | — | — | — |
| [`warps`](../../../../contracts/menus/warps.json) | list | travel | warps | daemon | `player.warp.list` | — |
