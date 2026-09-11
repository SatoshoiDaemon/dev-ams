# World Structure and Design

## World Structure

The game world is divided into distinct regions connected through an overworld/travel structure.

The world is NOT intended to be one seamless giant terminal map.

Instead, progression works through:

```text
Local Region
    ↓
Open World / Travel Area
    ↓
Region Boundary
    ↓
Destination Selection
    ↓
Next Connected Region
```

The player explores a region normally through terminal movement.

When traveling through the Open World and reaching a valid world boundary, the game presents a travel menu containing only regions that are directly connected to the player's current location.

The player cannot freely teleport to any discovered region unless a specific mechanic explicitly allows it.

World connectivity should therefore be represented as a graph rather than as a simple list.

Conceptually:

```text
Region A
 ├── Region B
 └── Region C

Region C
 └── Region D
```

Travel availability depends on adjacency, progression, routes, transportation, and special conditions.

## Region Design Philosophy

Every major region should have a gameplay identity.

A region should not exist only as a different background or enemy level.

Regions should differ through combinations of:

```text
enemies
resources
loot
glyphs
nodes
environmental mechanics
NPCs
quests
factions
build support
hazards
travel rules
campaign state
```

A player choosing where to go should be making a gameplay decision.

Regions may strongly favor certain builds, resources, mechanics, or progression paths without making those builds impossible elsewhere.

## Base World Topology

The exact final world graph may evolve.

The initial world should conceptually contain:

```text
                         The Ice Mountain
                               │
                               │
                        Open World Routes
                               │
       The Fairy Forest ─── Open World ─── Crimson Forest
               │               │
               │               │
           Tidal Town ───── Open World ─── The Golden Desert
               │
               │
            The Caves
               │
               │
         possible deeper routes

Tidal Town / Coastal Port
               │
             Ship
               │
            Riptide

Apexia
    connected through appropriate later world routes
```

This diagram expresses general relationships, not mandatory exact geography.

The final adjacency graph should be defined through world data.

## Known Regions

The current canonical major-region list is:

```text
Tidal Town
Open World
The Caves
The Fairy Forest
The Golden Desert
Crimson Forest
The Ice Mountain
Apexia
Riptide
```

Do not rename these canonical regions casually.

Internal stable IDs should remain separate from display names so names can be localized or adjusted without breaking saves.

## World System Rule

The central rule for world implementation is:

> Geography determines where the player can go. Progression changes what the world contains. Difficulty discourages exploration but does not automatically forbid it.

The world should feel connected rather than like a level-select screen, while still remaining practical for a terminal-based game.

The Open World, region graph, hidden routes, transportation systems, and persistent campaign state should work together to make travel itself part of progression.
