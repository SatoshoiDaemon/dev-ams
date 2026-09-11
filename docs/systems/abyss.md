# The Abyss

## Abyss

The Abyss is an infinite procedural dungeon.

It is primarily a late-game strength test.

Its central question is:

How strong has this character actually become?

The Abyss is not intended primarily as the optimal way to become powerful.

It is where an already powerful build is subjected to increasingly absurd generated opposition.

## Abyss Enemy Generation

The Abyss uses normal enemy templates as generation foundations.

Conceptually:

```text
Base Enemy Template
↓
Abyss Scaling
↓
Attribute Modification
↓
Equipment Assignment
↓
Weapon Generation
↓
Magic Generation
↓
Node Modification
↓
Final Abyss Enemy
```

The dungeon should reuse existing systems rather than relying only on fixed bespoke enemies.

## Abyss Procedural Builds

Generated Abyss enemies may receive:

- different attributes
- weapons
- armor
- magic
- Nodes
- effects
- Fighting Style interactions
- special modifiers

using the same systems available elsewhere in the game.

As depth increases, enemy builds may become increasingly extreme.

The purpose is not simply:

enemy HP × 10000

The procedural system should create mechanically different threats.

## Abyss Progression

The Abyss is infinite in principle.

Difficulty should continue increasing without requiring a predefined final floor.

Procedural scaling may modify:

- attribute budgets
- enemy count
- equipment quality
- Node complexity
- magic complexity
- AI behavior
- enemy combinations
- special modifiers

Avoid making infinite scaling depend solely on HP and damage inflation.
