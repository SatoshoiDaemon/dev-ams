# Attribute Scaling

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

Catalysts do not need to use only Spirit Scaling.

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
