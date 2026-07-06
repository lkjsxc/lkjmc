# Staff menu routes

## Purpose

This generated file lists `staff` menu routes from
[contracts/menus](../../../../contracts/menus).

## Status

implemented

## Routes

| Route | Kind | Parent | Binding | Source | Data commands | Confirmation |
| --- | --- | --- | --- | --- | --- | --- |
| [`admin`](../../../../contracts/menus/admin.json) | static | root | — | — | — | — |
| [`admin-audit`](../../../../contracts/menus/admin-audit.json) | list | admin | admin-audit | daemon | `admin.audit.tail` | — |
| [`admin-config`](../../../../contracts/menus/admin-config.json) | detail | admin | admin-config | daemon | `status` | — |
| [`admin-economy`](../../../../contracts/menus/admin-economy.json) | detail | admin | admin-economy | daemon | `player.shop.list` | — |
| [`admin-moderation`](../../../../contracts/menus/admin-moderation.json) | detail | admin | admin-moderation | daemon | `player.report.list` | — |
| [`admin-security`](../../../../contracts/menus/admin-security.json) | detail | admin | admin-security | daemon | `admin.role.list`, `security.daemon-token.status` | — |
| [`admin-server-create-confirm`](../../../../contracts/menus/admin-server-create-confirm.json) | confirm | admin-server-create-template | — | — | — | starts-durable-resources |
| [`admin-server-create-kind`](../../../../contracts/menus/admin-server-create-kind.json) | custom | admin-servers | admin-server-create-kind | daemon | `instance.create.plan` | — |
| [`admin-server-create-template`](../../../../contracts/menus/admin-server-create-template.json) | custom | admin-server-create-kind | admin-server-create-template | daemon | `instance.create.plan` | — |
| [`admin-server-detail`](../../../../contracts/menus/admin-server-detail.json) | detail | admin-servers | admin-server-detail | daemon | `instance.list` | — |
| [`admin-servers`](../../../../contracts/menus/admin-servers.json) | list | admin | admin-servers | daemon | `instance.list` | — |
| [`admin-web`](../../../../contracts/menus/admin-web.json) | detail | admin | admin-web | daemon | `status` | — |
