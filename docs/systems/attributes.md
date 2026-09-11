# Attributes, Power, and Defensive Resources

> **Unresolved naming decision:** The source design below calls the magical offensive attribute **Spirit** and says it supersedes Intelligence. Repository instructions define **Intelligence** and assign it magical damage, advanced Node access, and spell complexity/weight responsibilities. Both positions are preserved by this migration; it does not approve a rename or resolve the difference in responsibilities. Stable attribute IDs and save compatibility require an explicit decision before implementation.

## Core Attributes

Current core attributes are:

- Vigor
- Resistance
- Strength
- Dexterity
- Precision
- Spirit
- Mana

Localized attribute names, standardized into English:

- Vigor
- Resistance
- Strength
- Dexterity
- Precision
- Spirit
- Mana

Spirit is the primary magical offensive attribute in this version of the design.

If older documents/code refer to Intelligence as the magical offensive attribute, treat the current design as superseding that naming unless the maintainer explicitly decides otherwise.

## Vigor

Vigor increases HP.

Base relationship:

1 Vigor = +10 HP

This base value may be affected by configuration or other systems.

## Resistance

Resistance increases Tenacity.

Base relationship:

1 Resistance = +10 Tenacity

Tenacity also reduces the duration/effectiveness of relevant debuffs according to the configured system.

## Strength

Strength affects:

- physical strength
- unarmed damage
- Strength-scaling weapon damage
- Stamina spent on physical attacks

Strength generates offensive Power for Scaling.

Base relationship:

1 Strength = 7 Strength Power

## Dexterity

Dexterity affects:

- damage with most melee/body weapons
- evasion
- Priority

Dexterity generates Power.

Base relationship:

1 Dexterity = 7 Dexterity Power

## Precision

Precision affects:

- long-range weapon damage
- applicable ranged combat
- some magic

Precision generates Power.

Base relationship:

1 Precision = 7 Precision Power

## Spirit

Spirit is primarily responsible for magical offensive power.

It affects:

- magic damage
- magical Scaling
- potential magic-related requirements

Spirit generates Power.

Base relationship:

1 Spirit = 7 Spirit Power

Do not automatically make Spirit responsible for every magical subsystem.

Mana capacity and Node complexity remain separate mechanics.

## Mana

Mana increases the player's Mana/MP resource pool.

Base relationship:

1 Mana = +10 MP

Mana is separate from Spirit.

A character may therefore have:

high magical damage / low Mana

or:

low magical damage / large Mana reserve

depending on their build.

## HP and Tenacity

The player has two primary defensive bars:

- HP
- Tenacity

HP is primarily increased through Vigor.

Tenacity is primarily increased through Resistance.

While Tenacity remains active, applicable incoming damage is divided:

```text
50% → HP
50% → Tenacity
```

True Damage ignores Tenacity.

Tenacity additionally reduces debuff duration/effectiveness according to the appropriate rules.

Tenacity is not simply bonus HP.
