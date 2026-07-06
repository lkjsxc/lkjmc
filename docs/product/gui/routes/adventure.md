# Adventure menu routes

## Purpose

This generated file lists `adventure` menu routes from
[contracts/menus](../../../../contracts/menus).

## Status

implemented

## Routes

| Route | Kind | Parent | Binding | Source | Data commands | Confirmation |
| --- | --- | --- | --- | --- | --- | --- |
| [`adventures`](../../../../contracts/menus/adventures.json) | list | root | adventures | daemon | `adventure.catalog.list` | — |
| [`adventures-end-confirm`](../../../../contracts/menus/adventures-end-confirm.json) | confirm | adventures | — | — | — | starts-temporary-infrastructure |
| [`adventures-end-party-confirm`](../../../../contracts/menus/adventures-end-party-confirm.json) | confirm | adventures | — | — | — | affects-other-players |
