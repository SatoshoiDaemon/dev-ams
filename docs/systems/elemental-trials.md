# Elemental Trials and Awakening

## Elemental Trials

There is one Elemental Trial temple for each existing element.

These temples are spread throughout the world.

Each Elemental Trial has two primary modes.

## Elemental Trial — Acquisition

The first mode allows the player to obtain an element they do not currently possess.

Completing it grants:

```text
Elemental Affinity
+
Initial Nodes unique to that element
```

This is the primary progression method for expanding beyond the initial two character-creation elements.

The challenge should represent mastery or acceptance of the corresponding element.

## Elemental Trial — Awakening

The second mode is used when the player already possesses the corresponding element.

Completing the Awakening Trial:

- awakens the element
- increases its power
- unlocks more powerful Nodes
- reduces the weight of spells using that element

The reduced spell weight is a major mechanical reward.

This may improve:

- spell construction limits
- cast weight
- build efficiency
- Node combinations

depending on the final magic implementation.

## Awakened Elements

Awakening must be represented as persistent character progression.

Conceptually:

```text
Fire:
    unlocked = true
    awakened = true
```

Do not implement awakening as merely receiving one item.

Mods should eventually be capable of defining additional Trial behavior for modded elements.
