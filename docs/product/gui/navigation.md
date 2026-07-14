# Navigation

## Purpose

This document defines route, parent, Back, and close behavior.

## Status

implemented

## Behavior

`/menu` and the slot-8 token open `root`. `/docs` opens a documentation route in
the same engine. `NAVIGATE` validates target route and required parameters then
pushes the current route. `BACK` uses session history, falling back to the
document parent. Main Menu selects `root`. Refresh preserves route and history.

Navigation replaces the inventory directly. It never calls close, starts a
mutation, or loses route/session correlation. `CLOSE` is the only close action.
An explicit client close ends the session without treating it as navigation.

## Stale input

A click from an older render, route, session, or request is rejected and gives
localized fallback. It cannot navigate using stale row parameters.
