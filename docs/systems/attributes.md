# Attributes, Power, and Defensive Resources

> **Specification status:** The canonical section at the end is normative and defines every attribute calculation, cap, allocation, migration, and test value. Earlier text is historical where it conflicts.

> **Migration note:** `spirit` is a legacy save-data alias for the canonical `intelligence` attribute. It is not a current gameplay attribute.

## Core Attributes

Current core attributes are:

- Vigor
- Resistance
- Strength
- Dexterity
- Precision
- Intelligence
- Mana

Localized attribute names, standardized into English:

- Vigor
- Resistance
- Strength
- Dexterity
- Precision
- Intelligence
- Mana

Intelligence is the primary magical offensive attribute in this version of the design.

Older documents and save data may refer to this attribute as `Spirit`; the canonical ID is `intelligence` and the migration rule is defined below.

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

## Mana

Mana increases the player's Mana/MP resource pool.

Base relationship:

1 Mana = +10 MP

Mana is separate from Intelligence.

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

## Canonical Quantified Attribute Contract

The stable attribute IDs are `vigor`, `resistance`, `strength`, `dexterity`, `precision`, `intelligence`, and `mana`. `spirit` is not a separate attribute; old save data using `spirit` migrates one-for-one into `intelligence`, with a migration warning in the log. Level 1 provides `100 Max HP`, `50 Max Tenacity`, `30 Max Mana`, and `0` points in each attribute. Each level grants `1` attribute point; every 5th level grants one additional point.

Derived values are:

```text
Max HP       = 100 + 10 × Vigor
Max Tenacity = 50 + 10 × Resistance
Max Mana     = 30 + 10 × Mana
Attribute Power = 7 × attribute points
Priority     = 100 + 2 × Dexterity + equipment_priority + status_priority
```

Strength Power scales physical/unarmed and Strength-tagged attacks; Dexterity Power scales melee/body weapons and contributes Evasion; Precision Power scales ranged weapons and Precision-tagged spells; Intelligence Power scales Magical damage and Spell Capacity. Power is never added directly to damage unless a Scaling Grade consumes it.

The global Scaling Grade multipliers are D `0.25`, C `0.50`, B `0.75`, A `1.00`, S `1.50`, SS `2.00`, SSS `2.50`. For each scaling entry: `floor(Attribute Power × grade multiplier)` is calculated first, then entries are summed. A single attribute may receive enhancement from at most 4 distinct sources; a fifth source is ignored and logged with source IDs.

Secondary curves use their owning mechanics. Evasion uses the canonical Evasion Rating contract in [Combat](combat.md#accuracy-and-evasion), and Resistance debuff reduction is `final_counter = max(1, floor(base_counter × 1000 / (1000 + Resistance × 10)))`. The engine does not impose a generic Damage Reduction balance cap. Attributes have no gameplay hard maximum, but a save rejects values below zero and values above `1_000_000` with a validation error for data safety.

Worked examples: a character with Vigor 12, Resistance 5, Mana 8 has `220 HP`, `100 Tenacity`, and `110 Mana`; Strength 10 at B Scaling contributes `floor(70 × .75) = 52`; Intelligence 20 gives `140 Intelligence Power` and Spell Capacity `20 + 2 × 20 = 60` before Catalyst and Grimoire bonuses. Tests must cover all seven IDs, Spirit migration, source limit, floor rounding, zero/maximum validation, and tenacity/debuff reduction.
