# Claims menu routes

## Purpose

This generated file lists `claims` menu routes from
[contracts/menus](../../../../contracts/menus).

## Status

implemented

## Routes

| Route | Kind | Parent | Binding | Source | Data commands | Confirmation |
| --- | --- | --- | --- | --- | --- | --- |
| [`claim-create-confirm`](../../../../contracts/menus/claim-create-confirm.json) | confirm | claims | — | — | — | creates-durable-world-state |
| [`claim-detail`](../../../../contracts/menus/claim-detail.json) | detail | claims | claim-detail | daemon | `claim.snapshot` | — |
| [`claim-trust-picker`](../../../../contracts/menus/claim-trust-picker.json) | list | claim-detail | claim-trust-picker | local | — | — |
| [`claims`](../../../../contracts/menus/claims.json) | list | root | claims | daemon | `claim.list` | — |
