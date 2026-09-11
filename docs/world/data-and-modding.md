# World Data and Modding

## World Data

Regions should be data-driven whenever practical.

Avoid hardcoding all world connections directly into Rust.

A region definition should be able to describe concepts such as:

```text
ID
display name
region type
connections
entry requirements
hidden entrances
travel method
enemy pools
resource pools
loot tables
environmental rules
NPCs
factions
campaign state changes
```

Conceptual example:

```json
{
  "id": "base:tidal_town",
  "name": "Tidal Town",
  "type": "city",
  "connections": [
    "base:open_world_coast"
  ]
}
```

This is illustrative, not a mandatory final schema.

World topology should remain moddable.

Mods should eventually be capable of:

```text
adding new regions
adding routes
blocking routes
creating hidden routes
adding transport methods
changing encounters
changing resource tables
changing regional states
```

## World Modding

World systems must be mod-friendly.

Mods should eventually be able to register:

```text
regions
sub-regions
routes
hidden entrances
encounter tables
resource tables
environmental rules
transportation routes
regional NPCs
world events
regional state transitions
```

Lua should be capable of controlling dynamic behavior.

Example conceptual API:

```text
world.register_region(...)
world.register_route(...)
world.set_route_available(...)
world.discover_route(...)
world.get_region_state(...)
world.set_region_state(...)
world.on_enter(...)
world.on_leave(...)
```

These names are illustrative.

Do not expose world modding solely through giant hardcoded enums.
