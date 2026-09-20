# Levels and Build Progression

> **Specification status:** The progression values in [Numeric Specification Baseline](../dev/numeric-specification-baseline.md) are canonical. The examples and philosophy below are descriptive unless they conflict with that baseline.

## Experience and Levels

Character level progression uses a linear XP-growth model.

Current intended formula:

- XP Required =
- (Current Level × 1000) + 100

Example:

```text
Level 1
→ 1,100 XP
```

```text
Level 10
→ 10,100 XP
```

```text
Level 100
→ 100,100 XP
```

The exact formula should remain centralized/configurable.

Do not duplicate the formula across unrelated systems.

## Maximum Level

Maximum intended level:

7000

The purpose is to support eventual allocation of approximately:

1000 points into every core attribute

assuming the final progression/allocation model matches that structure.

Level 7000 is intended as an extreme long-term ceiling.

The game should not require normal campaign completion to reach maximum level.

## Attribute Progression Philosophy

The game intentionally allows extremely high attribute values.

Therefore, mathematical systems must support large numbers without becoming either:

completely broken

or:

effectively meaningless

Soft caps and diminishing returns are not global balance requirements. A specific mechanic may define one when its own behavior requires it.

However:

Additional investment must continue producing a meaningful benefit.

Avoid curves where an additional point eventually produces absurdly tiny results such as:

```text
+1 attribute
→ 0.05% relevant improvement
```

unless that behavior is explicitly intended for that particular mechanic.

## Hard Limits

Some derived mechanics require asymptotic or hard maximums.

Examples:

Evasion must never reach 100%.

Damage Reduction must never reach 100%.

Similar defensive or avoidance mechanics may require equivalent caps.

The player must not become literally untouchable through a generic percentage stat unless a specific ability explicitly grants such a state.

The system must not add diminishing returns or configured maximums solely to contain balance. A specific mechanic may define a structural limit when required by its rules.

## Progression Sources

Character strength should come from multiple systems.

Examples:

- Level
- Attributes
- Equipment
- Weapon Levels
- Scaling
- Nodes
- Glyphs
- Fighting Styles
- Elements
- Element Awakening
- Templates
- Crafting
- Alchemy
- Trinkets
- Talismans
- Shrine Rewards
- Arena Rewards
- Legendary Equipment

No single system should necessarily represent the entire character progression curve.

This is fundamental to build variety.

## Build Freedom

The player should be able to combine systems in unusual ways.

Examples include:

- high Dexterity heavy weapon user
- martial caster
- ranged Fighting Style hybrid
- status-focused melee build
- high-Mana low-Intelligence utility caster
- high-Intelligence low-Mana burst caster
- critical spell build
- blood/debuff build
- weapon-focused mage

Do not add class restrictions simply because a particular combination is unconventional.

Mechanical costs should determine whether a build is practical.

## Natural Restrictions

Prefer systemic restrictions.

Examples:

Instead of:

Mage cannot equip Greatsword.

prefer:

- Greatsword has high Strength requirements
- high weight
- particular Scaling
- specific Node interactions

Instead of:

You cannot cast this because your class is wrong.

prefer:

- You lack the Element
- Node access
- Mana
- appropriate catalyst
- spell weight capacity

Restrictions should emerge from the actual systems whenever possible.
