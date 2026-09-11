# World State, Campaign Changes, and Save Compatibility

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

Unknown modded regions or routes should generate useful diagnostics rather than unexplained crashes.
