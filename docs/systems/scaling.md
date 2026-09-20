# Attribute Scaling

> **Specification status:** The canonical contribution rule below is normative. Earlier sections preserve the original design vocabulary and content proposals.

See [attributes](attributes.md) for Power generation, [Fighting Styles](fighting-styles.md) for style progression, and [Glyphs](glyphs.md) for proposed catalyst Glyph specializations. This source describes Scaling structure and grade availability; the central engine rules remain authoritative for multipliers and damage-pipeline order.

## Scaling System

Scaling represents how strongly an item, spell, ability, or style benefits from an attribute.

Entities that may possess Scaling include:

- weapons
- catalysts
- spells
- Arts of War
- Fighting Styles

Each relevant Scaling associates:

```text
Attribute
+
Scaling Grade
```

## Scaling Grades

Current intended grade range for standard Scaling:

- D
- C
- B
- A
- S

Core multipliers currently defined elsewhere may support additional grades such as:

- SS
- SSS

If those grades remain part of the global Scaling system, systems consuming Scaling must not assume S is necessarily the technical maximum.

The content layer may restrict ordinary gear to D-S while exceptional content uses higher grades.

## Weapon Scaling

Weapon Scaling is influenced by factors such as:

- base material
- weapon template
- weapon level
- Nodes
- modifications
- special properties

Scaling should therefore be calculated from a weapon definition and its current state rather than stored only as one immutable enum.

Different materials may naturally favor different attributes.

## Catalyst Scaling

Catalysts are treated as weapons/equipment for Scaling purposes.

Catalyst Scaling influences relevant magical output.

Their Scaling may depend on:

- material
- template
- level
- Nodes
- element
- special properties

Catalysts do not need to use only Intelligence Scaling.

Specialized catalysts may scale from other attributes where the content defines it.

## Spell Scaling

Spells possess Scaling independently of their catalyst.

This permits interactions such as:

```text
Spell Scaling
+
Catalyst Scaling
+
Nodes
+
Effects
```

The exact formula must respect the central damage pipeline.

Do not merge all magical Scaling into one hidden multiplier.

## Art of War Scaling

Arts of War may possess their own Scaling.

Their Scaling represents how strongly the technique benefits from specific attributes.

The architecture should allow multi-attribute Scaling where defined.

## Canonical Scaling Contribution Contract

Each Scaling source associates one attribute with one Scaling Grade. The authoritative grade multipliers are:

```text
D = 0.25
C = 0.50
B = 0.75
A = 1.00
S = 1.50
SS = 2.00
SSS = 2.50
```

For every declared source:

```text
scaling_contribution = floor(relevant_attribute_power × grade_multiplier)
```

Multiple sources are additive:

```text
pre_mitigation_damage =
    flat_base_damage + sum(scaling_contributions)
```

Power is not added directly to damage. A source without a declared Scaling does not receive an implicit attribute contribution. Scaling contributions are calculated before offensive modifiers and Defense/Block/Shield/Tenacity/HP processing.

Example:

```text
Flat Base Damage = 40
Strength Power = 80
Strength Scaling = B (0.75)

Contribution = floor(80 × 0.75) = 60
Pre-Mitigation Damage = 40 + 60 = 100
```

For multiple Scalings:

```text
Strength Power = 80, B = 60
Dexterity Power = 35, C = 17
Flat Base Damage = 20

Pre-Mitigation Damage = 20 + 60 + 17 = 97
```

The action log must preserve each source ID, attribute ID, Power value, grade, multiplier, and floored contribution. Catalyst, spell, weapon, Glyph, and Fighting Style Scaling may coexist only when the owning content definition declares them.
