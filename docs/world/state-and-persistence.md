# World State, Campaign Changes, and Save Compatibility

> **Canonical save philosophy:** Save files are player-owned data. Manual editing
> of JSON inside a save ZIP is supported. The engine checks structural safety, not
> gameplay legitimacy. The shared ownership, migration, and opaque mod-data rules
> are defined in [Technical Contracts](../dev/technical-contracts.md#save-ownership-and-migrations).

## World State

Regions must support persistent world state.

Examples:

```text
Fairies hostile/friendly
route discovered/undiscovered
bridge intact/destroyed
pirate target alive/dead
merchant available/unavailable
cave opened/collapsed
city faction hostile/friendly
```

World state belongs to the save.

It should not be stored only in global `config.toml`.

Conceptually:

```text
Global configuration
→ defines rules

Save world state
→ defines what happened in this campaign
```

World-state entries should use stable IDs to remain compatible with mods and save editing.

## Campaign Changes

Campaign progression may alter existing regions instead of only unlocking new ones.

The Fairy Forest faction transition is the first explicit example.

Future campaign changes may alter:

```text
enemy tables
NPCs
shops
factions
routes
environment
settlements
loot
quests
region ownership
```

Avoid architectures where regions are loaded once as immutable definitions and can never evolve.

Use:

```text
Region Definition
+
Save World State
+
Campaign State
=
Current Region State
```

## Save Compatibility

Persistent world information should use stable IDs.

Avoid storing only array positions such as:

```text
region = 4
```

Prefer:

```text
region = "base:golden_desert"
```

Likewise:

```text
discovered_routes = [
    "base:fairy_forest_hidden_entrance"
]
```

This makes saves:

```text
human-readable
editable
shareable
more resilient to content changes
mod-friendly
```

Unknown modded regions or routes should generate useful diagnostics rather than
unexplained crashes. If they are mod-owned save data, their opaque JSON must be
preserved. If an active world state references an unavailable content ID, represent
it as an unresolved reference and let the owning world operation decide whether
that particular operation can continue; do not automatically invalidate the whole
save.
