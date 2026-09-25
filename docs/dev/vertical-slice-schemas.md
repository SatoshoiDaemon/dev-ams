# Vertical Slice Schemas

> **Status:** Implemented schema reference for ruleset `base:standard@1`, save
> format 2, and mod API 1.

## Base content package

`data/manifest.json` identifies the first-party package. `data/content/` contains
JSON arrays for `actions`, `statuses`, `entities`, `encounters`, and
`spell_templates`. Files are loaded before third-party mods and cross-references are
validated after the registries exist. Duplicate IDs fail unless the incoming action
or status explicitly uses `replacement_of` for the existing same ID.

An action defines:

```text
id, replacement_of?, source_id
target { mode, relation, state, filters[] }
ap_cost, mana_cost, cast, action_priority
availability
properties { offensive, blockable, interruptible, bonus_eligible,
             magical, weapon_dependent, refundable, reaction_power? }
requirements[], tags[], steps[]
```

Supported step kinds are `attack`, `direct_damage`, `apply_status`,
`remove_status`, `heal`, `shield`, `change_mana`, and `flee`. Attack Scaling is an
array of `{ attribute, grade }`; Power is calculated by the ruleset and never used
as raw damage.

A status defines identity, categories, Potency/Counter bounds, optional fixed
Potency, tick and decay behavior, modifiers, tags, immunity tags, and suppressed
categories. Engine primitives are Rust code; each base effect's values and behavior
selection live in JSON.

Entity templates contain stable identity, allegiance, race/tags, attributes,
Defense, and action IDs. Encounter definitions map stable instance IDs to entity
templates. Spell templates contain race ownership, Node IDs, Glyph IDs, and an
optional demonstration action.

## Mods

Each directory below `mods/` may contain:

```text
manifest.json
content/actions.json
content/statuses.json
scripts/*.lua
```

The manifest fields are `id`, `name`, `version`, `api_version`, and
`dependencies[]`. Discovery and script order are path-stable. A bad mod is reported
without stopping independent mods; its dependents are blocked.

Lua exposes `mod_api_version = 1` and the `game` table. Registration calls are
load-time only. Queries (`get_entity`, `get_attribute`, `get_resource`,
`has_status`) return copied values. Mutations (`damage`, `heal`,
`change_resource`, `apply_status`, `remove_status`) enqueue `EngineCommand`; Lua
never receives Rust references. `on(event, listener_id, callback)` requires a
namespaced stable listener ID. A failing callback is disabled for that session.

## Game modes and rulesets

Every TOML file in `gamemodes/` declares an internal `id` and `ruleset_version`.
Its optional `[numeric]` keys override `config.toml`; the filename has no semantic
meaning. A new game resolves compiled safe defaults, global configuration, then the
selected gamemode. A loaded v2 game uses its saved `ruleset.json` snapshot last.

## Save format 2

```text
metadata.json  format/ruleset/gamemode, required mods, player IDs, RNG state
ruleset.json   resolved rules snapshot
entities.json  human-readable entity state
combat.json    optional complete CombatSession and explain-last packet
mod_data.json  opaque JSON subtrees keyed by mod namespace
```

V1 archives are migrated as non-combat saves. Entities and opaque mod data are
preserved, the currently referenced ruleset is selected, and the loader emits a
warning because v1 had no ruleset snapshot. Save names are 1–64 ASCII letters,
digits, `_`, or `-`; the resulting archive always remains under `saves/`.

## CLI

Menu: `help`, `new`, `load`, `saves`, `quit`.

Ready: `status`, `combat base:training-encounter`, `save`, `explain last`,
`log normal|verbose`, `menu`, `quit`.

Combat: `status`, `actions`, `use`, `reserve`, `end`, `save`, `explain last`,
`flee`, `quit`.
