# Character Creation, Races, and Element Ownership

The provisional narrative background is documented in [world lore](../world/lore.md). Further elemental progression is defined in [Elemental Trials](elemental-trials.md).

## Character Creation

At the beginning of the game, the player creates their character.

The two primary initial decisions are:

```text
Race
+
Initial Elements
```

These decisions should affect gameplay immediately.

## Races

Initial playable races:

- Human
- Demon
- Angel
- Elf
- Feral
- Dragonborn

Each race MUST eventually have a mechanically distinct passive, resource interaction, transformation, progression mechanic, or other defining system.

Race differences should not be purely cosmetic.

The exact racial mechanics are currently undefined and should not be invented without explicit design decisions.

The architecture must therefore support race definitions with configurable/moddable mechanics.

Conceptually, races may expose:

- base attributes
- passives
- resource modifiers
- element interactions
- equipment restrictions
- special progression
- Lua hooks

The base implementation should not assume all races behave identically.

## Initial Elements

During character creation, the player may choose up to:

2 Elements

Initial elemental options:

- Fire
- Water
- Earth
- Air
- Light
- Darkness
- Poison
- Ice
- Lightning

These initial elements determine:

- which elemental Nodes the player can initially access;
- which elemental magic structures are initially available;
- which early spells the player can construct or use;
- which elemental progression paths begin unlocked.

The player's initial element selection does NOT permanently prevent access to other elements.

Additional elements can later be obtained through Elemental Trials.

## Element Ownership

Elements are persistent character progression.

A character can eventually own more elemental affinities than the two selected during character creation.

The game should distinguish between at least:

- Element unavailable
- Element unlocked
- Element awakened

Additional progression tiers may be added later.

Element state belongs to the save.

Element IDs should remain stable and moddable.
