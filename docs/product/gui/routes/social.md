# Social menu routes

## Purpose

This generated file lists `social` menu routes from
[contracts/menus](../../../../contracts/menus).

## Status

implemented

## Routes

| Route | Kind | Parent | Binding | Source | Data commands | Confirmation |
| --- | --- | --- | --- | --- | --- | --- |
| [`mail`](../../../../contracts/menus/mail.json) | list | social | mail | daemon | `player.mail.inbox` | — |
| [`party`](../../../../contracts/menus/party.json) | custom | social | party | daemon | `player.party.info` | — |
| [`party-confirm`](../../../../contracts/menus/party-confirm.json) | confirm | party | — | — | — | affects-other-players |
| [`party-invite-picker`](../../../../contracts/menus/party-invite-picker.json) | list | party | party-invite-picker | local | — | — |
| [`report-detail`](../../../../contracts/menus/report-detail.json) | detail | reports | report-detail | daemon | `player.report.list` | — |
| [`reports`](../../../../contracts/menus/reports.json) | list | social | reports | daemon | `player.report.list` | — |
| [`social`](../../../../contracts/menus/social.json) | static | root | — | — | — | — |
