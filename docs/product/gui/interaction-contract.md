# Menu interaction contract

## Metadata

Every rendered item carries route id, session id, render revision, slot, and
action id. The listener cancels top-inventory clicks before decoding. An empty
slot, changed persistent metadata, route mismatch, stale session, stale render,
or unknown action has no navigation effect and returns localized failure where
appropriate.

## Session behavior

Navigation and Back replace the inventory without treating the replacement close
event as player closure. Only `CLOSE` closes intentionally. Reopen replaces the
prior per-player adapter. Close, quit, locale change, and plugin disable retire
ownership.

The action set is closed to `NAVIGATE`, `BACK`, `CLOSE`, and `NONE`. Authored
route slots use only navigation and inert information; Back and Close come from
route chrome. There is no pending request, remote response, refresh, mutation,
confirmation, capability, or attestation path in this menu.

## Threading

Menu handlers only validate metadata, render bundled data, and apply Bukkit
inventory effects. They do not wait on database, filesystem, network, process,
download, or worker completion.
