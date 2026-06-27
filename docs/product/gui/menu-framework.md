# Menu framework

## Purpose

This document defines the reusable platform-neutral inventory UI framework.

## Pure domain

Common Java owns records or sealed types for menu ids, routes, route params,
route stacks, navigation state, menu specs, surfaces, titles, sizes, slots,
item specs, lore, visual roles, themes, actions, payloads, metadata, state,
context, clicks, decisions, effects, failures, pagination, page windows,
navigation policy, registry, renderer models, and dynamic menu models.

## Required actions

Actions include none, open route, back, close, refresh route, run player command,
daemon command, transfer, confirm, disabled, select, purchase, and toggle.
`OpenRoute` is forward navigation or an explicit shortcut; `Back` is historical
stack pop. The payload is opaque to the framework and is interpreted only by the
action owner.

## Required effects

Effects include open route, open previous, close menu, refresh route, run player
command, send daemon command, transfer player, send message, render loading then
run, and noop.

## Reducer rules

`render(spec, context, dynamicData)` returns a renderer model. `click(spec,
state, click)` returns a decision. Pure navigation functions own stack
invariants: root is the bottom route, Back never pushes, OpenRoute pushes only
when distinct, and refresh preserves the stack. Reducers do not import Bukkit,
Paper, Folia, Velocity, network, database, filesystem, or process APIs. Reducers
classify empty, inert, stale, mismatched, disabled, navigation, and real action
clicks.

## Adapter boundary

Adapters execute effects, load dynamic data asynchronously, write metadata, and
schedule game mutations. They delegate route-stack changes to common pure code
and keep dynamic loading or unavailable replacement on the current stack. Common
code remains deterministic and testable.
