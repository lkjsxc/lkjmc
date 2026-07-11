# Menu documents

## Purpose

`contracts/menus/*.json` is the structural source of truth for planned menu
routes. One route file defines one route, and the filename must equal the
route id. The generated `README.json` file is a resource index for JVM loading,
not a route document.

## File format

A menu document is a JSON object:

```json
{
  "id": "shop",
  "kind": "list",
  "title": "menu.shop.title",
  "theme": "economy",
  "size": 54,
  "params": [{"name": "category", "required": false}],
  "parent": "economy",
  "data": {"binding": "shop", "source": "daemon"},
  "chrome": {"info": "menu.shop.info", "back": true,
    "refresh": true, "close": true, "mainMenu": true},
  "list": {"region": "interior-21", "reserved": "filter-row",
    "pagination": true, "emptyName": "menu.shop.empty",
    "emptyLore": ["menu.shop.empty.lore"]},
  "static": [],
  "confirmation": null
}
```

## Field vocabulary

- `id`: lowercase kebab-case, unique, and equal to filename.
- `kind`: `static`, `list`, `detail`, `confirm`, or `custom`.
- `title`: locale key; dynamic values belong in info lore.
- `theme`: `root`, `network`, `travel`, `claims`, `economy`, `social`,
  `profile`, `settings`, `staff`, `adventure`, `danger`, or `docs`.
- `size`: supported inventory size for the kind; compact confirm routes use
  `27`, normal routes use `54`.
- `params`: declared route params with `name` and `required`.
- `parent`: docs and reachability hint only; runtime Back uses live history.
- `data`: absent for static routes; otherwise `binding` plus source `daemon` or
  `local`.
- `chrome`: booleans for `back`, `refresh`, `close`, `mainMenu`, plus optional
  `info` locale key.
- `list`: list grammar with `region`, optional `reserved`, `pagination`, and
  true-empty copy keys.
- `static`: positioned static slots.
- `confirmation`: `null` or a reason token from the owner policy.

## Kind rules

`static` documents are fully described by static slots. `list` documents place
binding entries into a named region and let the kernel paginate. `detail`
documents place binding-provided slots. `confirm` documents use two-button
confirmation and require a confirmation reason. `custom` documents name a small
kernel view extension such as `docs-file` or the admin create wizard.

## Static slot format

```json
{
  "slot": 20,
  "material": "ENDER_PEARL",
  "name": "menu.travel.title",
  "lore": ["menu.travel.lore"],
  "role": "navigation",
  "action": {"type": "open", "route": "travel"}
}
```

`role` is `info`, `action`, `navigation`, `decoration`, `disabled`, `success`,
or `danger`. Role controls styling only.

## Action grammar

Static slots may use these actions:

- `{"type": "open", "route": "travel", "params": {"k": "v"}}` opens a route.
  Param values may forward current params with `@param.<name>`.
- `{"type": "back"}`, `{"type": "close"}`, and `{"type": "refresh"}` map to
  chrome actions.
- `{"type": "command", "value": "docs"}` runs a player command and keeps the
  session unless the command opens another surface.
- `{"type": "daemon", "command": "player.settings.set", "body": {},
  "ok": "language.saved", "fail": "language.failed", "refresh": true}` sends
  a daemon command. Body values may use `@player.uuid`, `@player.name`, or
  `@param.<name>`. Feedback fields are locale keys.
- `{"type": "input", "prompt": "menu.homes.name.prompt",
  "commandPrefix": "sethome"}` prompts for one chat line with 60-second expiry.
  Prefixes are command literals without a trailing space; the kernel inserts
  one separator before the submitted text.
- `{"type": "message", "key": "docs.external-link",
  "args": {"url": "@param.url"}}` sends a catalog message with args.
- `{"type": "none"}` is inert and required for info and decoration roles.

Dynamic entries receive typed actions from bindings, not from JSON.

## Region names

`RegionCatalog` and this contract define the same names:

- `interior-28`: slots `10-16`, `19-25`, `28-34`, `37-43`.
- `interior-21`: slots `19-25`, `28-34`, `37-43`.
- `filter-row`: slots `10-16`.
- `detail-band`: slots `20-24`.
- `confirm-pair`: slots `11` and `15`.

Documents reference region names only; they do not repeat slot arrays.

## Root document

`root.json` is a `static` entrypoint document. It uses info `4`, Network `19`,
Travel `20`, Claims `21`, Economy `22`, Social `23`, Profile `24`, Settings
`25`, Documentation `30`, Admin `31`, Adventures `40`, and Close `53`.
Documentation opens `docs-directory` with `path=docs`.

## Confirmation reasons

Allowed values are `deletes-durable-state`, `overwrites-named-durable-state`,
`creates-durable-world-state`, `writes-named-durable-state`, `stops-server`,
`forceful-server-mutation`, `starts-durable-resources`,
`starts-temporary-infrastructure`, `affects-other-players`,
`changes-moderation-state`, and `paid-dimension-change`.

## Validation

`check-menus.py` verifies JSON shape, ids, kinds, slot bounds, chrome
collisions, duplicate slots, region names, confirmation reasons, locale keys,
daemon command names and Paper surface membership, route targets, declared
params, required param forwarding, reachability, and generated route-doc parity.

## Golden evidence boundary

Structural validation does not prove a player sees a stable result when a menu
request succeeds or fails. The root, admin, server, shop, docs, and settings
flows need reviewed golden frames covering their visible success and failure
states. `F-CLAIM-PROBES` records the absence of that evidence as a rejected
shape only; it does not add placeholder documents or promote a playable claim.

## Change procedure

1. Edit or add the menu document.
2. Update locale keys in `config/locales/en.json` and `config/locales/ja.json`.
3. Ensure daemon commands already exist in `contracts/commands.json` when a
   daemon action uses them.
4. Run `scripts/check-menus.py`.
5. Regenerate route docs and the JVM resource index with
   `scripts/generate-menu-docs.py`, then commit the generated route catalog.

Do not add fake menu documents for routes without real bindings, commands, or
runtime support.
