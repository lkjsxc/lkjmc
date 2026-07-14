# Adventure menu routes

## Purpose

This generated file lists `ADVENTURE` menu routes from
[contracts/menus](../../../../contracts/menus).

## Status

implemented

## Routes

| Route | Kind | Parent | Binding | Source | Data commands | Confirmation |
| --- | --- | --- | --- | --- | --- | --- |
| [`adventures`](../../../../contracts/menus/adventures.json) | LIST | root | — | — | — | — |
| [`adventures-end-confirm`](../../../../contracts/menus/adventures-end-confirm.json) | CONFIRM | adventures | — | — | — | starts-temporary-infrastructure |
| [`adventures-end-party-confirm`](../../../../contracts/menus/adventures-end-party-confirm.json) | CONFIRM | adventures | — | — | — | affects-other-players |
