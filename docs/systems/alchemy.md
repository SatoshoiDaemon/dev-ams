# Alchemy and Dynamic Potions

> **Specification status:** The canonical numeric contract is defined in [Numeric Specification Baseline](../dev/numeric-specification-baseline.md). Potion duration is represented by Status Effect Counter; there is no independent generic runtime Duration. Earlier terminology is retained as historical vocabulary only.

## Alchemy

Alchemy is an experimental ingredient-based crafting system.

Potions are NOT primarily fixed recipes selected from a menu.

Instead:

```text
Ingredients
↓
Combination
↓
Alchemy Calculation
↓
Success / Failure
↓
Generated Potion
```

The resulting potion is determined by the ingredients used.

Ingredients may come from:

- plants
- chests
- enemies
- regional resources
- rare materials

## Alchemy Success

Potion creation is not instantaneous guaranteed crafting.

Alchemy has a success chance.

When creation fails:

Ingredients are lost.

The player improves their practical reliability with particular combinations through repeated experimentation.

Using the same ingredients and/or producing the same potion repeatedly increases the probability of successfully creating it again.

This represents accumulated alchemical familiarity.

Conceptually:

Recipe Familiarity

should be stored as character/save progression.

The exact success formula must remain configurable.

## Dynamic Potions

Potions have generated properties rather than only fixed predefined identities.

Relevant properties include:

- Effect
- Intensity
- Potency
- Duration
- Modifiers

Different ingredient combinations may alter these properties.

The alchemy system should preserve enough information to reconstruct why a potion has its resulting behavior.

Repeatedly creating a known combination should not prevent experimentation with variations.
