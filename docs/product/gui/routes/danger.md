# Danger menu routes

## Purpose

This generated file lists `danger` menu routes from
[contracts/menus](../../../../contracts/menus).

## Status

implemented

## Routes

| Route | Kind | Parent | Binding | Source | Data commands | Confirmation |
| --- | --- | --- | --- | --- | --- | --- |
| [`admin-server-delete-confirm`](../../../../contracts/menus/admin-server-delete-confirm.json) | confirm | admin-server-detail | — | — | — | deletes-durable-state |
| [`admin-server-restart-confirm`](../../../../contracts/menus/admin-server-restart-confirm.json) | confirm | admin-server-detail | — | — | — | forceful-server-mutation |
| [`admin-server-stop-confirm`](../../../../contracts/menus/admin-server-stop-confirm.json) | confirm | admin-server-detail | — | — | — | stops-server |
| [`claim-confirm`](../../../../contracts/menus/claim-confirm.json) | confirm | claim-detail | — | — | — | deletes-durable-state |
| [`home-delete-confirm`](../../../../contracts/menus/home-delete-confirm.json) | confirm | home-detail | — | — | — | deletes-durable-state |
| [`home-update-confirm`](../../../../contracts/menus/home-update-confirm.json) | confirm | home-detail | — | — | — | overwrites-named-durable-state |
| [`random-teleport-end-confirm`](../../../../contracts/menus/random-teleport-end-confirm.json) | confirm | travel | random-teleport | daemon | `player.random-teleport.quote` | paid-dimension-change |
| [`random-teleport-nether-confirm`](../../../../contracts/menus/random-teleport-nether-confirm.json) | confirm | travel | random-teleport | daemon | `player.random-teleport.quote` | paid-dimension-change |
| [`report-confirm`](../../../../contracts/menus/report-confirm.json) | confirm | report-detail | — | — | — | changes-moderation-state |
