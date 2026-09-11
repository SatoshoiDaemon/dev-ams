# Combat

Related rules: [status effects](effects.md), [attributes and resources](attributes.md), [Scaling](scaling.md), [Fighting Styles](fighting-styles.md), and [equipment](equipment.md).

## Critical Damage

Critical damage exists as an explicit mechanic.

There is NO mandatory universal random critical-hit chance.

Weapons, spells, Fighting Styles, Nodes, effects, and specific mechanics may create critical damage behavior.

A weapon may therefore support critical damage without randomly critting by default.

Critical hits may be activated through conditions such as:

- specific attacks
- enemy state
- position
- weapon properties
- Fighting Style
- Nodes
- effects
- bonus actions
- guaranteed conditions

Do not introduce a global random critical chance unless explicitly added later.

Critical damage itself should remain configurable and moddable.

## Combat Entry

Exploration and combat are separate interaction states.

When the player enters combat:

```text
Exploration
↓
Combat Initialization
↓
First Round
```

The player ALWAYS receives the first action opportunity when combat begins.

This first-action rule is an explicit combat-opening advantage.

After the opening action/round condition has been satisfied, normal Priority rules apply.

Do not interpret this as the player permanently having first Priority every round.

## Round Structure

Combat uses closed rounds.

Each round follows this general sequence:

```text
Round Start
↓
All actors declare actions
↓
Priority Calculation
↓
Action Queue Construction
↓
Actions Resolve
↓
End-of-Round Effects
↓
Counters / Potencies / Durations Update
↓
Next Round
```

Action declaration happens BEFORE action resolution.

This is important.

Actors commit to actions without necessarily knowing every action's final resolution order.

## Action Declaration

Every combat actor with active actions declares an action for the current round.

After declarations are completed, Priority is calculated.

This means a player may choose a powerful but slow action and then discover that multiple faster actions resolve before it.

The engine should preserve declared actions as explicit combat objects rather than immediately executing each declaration.

## Priority

Priority determines action resolution order.

Priority may be influenced by:

- Dexterity
- Haste
- Exhaust
- Equipment Weight
- Spell Cast Time
- Action Weight
- Equipment Modifiers
- Effects
- Fighting Styles
- Nodes

The exact numerical formula should be configurable.

Conceptually:

```text
Base Priority
+
Dexterity Contribution
+
Haste
-
Exhaust
-
Action Cost
-
Cast Time
-
Weight
+
Other Modifiers
=
Final Priority
```

This is illustrative, not the final required formula.

## Priority as a Build Mechanic

Priority is not merely a hidden initiative roll.

It is an active gameplay mechanic.

Powerful abilities may deliberately carry Priority penalties.

For example:

```text
Prepare devastating spell
↓
High cast weight
↓
Low Priority
↓
Action resolves near the end of the round
```

Players may build around:

- acting early
- interrupting slow enemies
- accepting slow actions for extreme damage
- Haste
- reducing equipment penalties
- increasing Dexterity

Avoid randomizing action order unnecessarily.

Priority should be explainable.

## Independent Combat Actors

Every entity with its own active actions receives its own combat turn/action declaration.

This includes:

- player
- enemies
- active companions
- active pets
- summons
- other controlled actors

An entity that only provides passive effects does NOT receive a turn.

Do not artificially merge every player-controlled entity into one action.

## Player Combat Actions

The player's default combat action categories are:

- Use Weapon / Equipment
- Use Magic
- Use Potion
- Access Inventory
- Flee

Additional actions may be added by:

- items
- equipment
- Nodes
- Lua
- mods
- specific encounters

## Potions in Combat

The player may carry up to:

3 Quick-Access Potions

Using a potion from quick access:

does NOT consume the player's normal turn.

Changing the quick-access potion selection during combat:

consumes the player's turn.

This distinction is intentional.

Prepared consumables reward planning before combat.

## Magic in Combat

The player may cast magic from:

- equipped catalysts
- personally carried/known spells

Catalysts may provide their own magic access and scaling.

Spell construction, Nodes, elemental ownership, cast weight, Mana usage, and Priority interact with this system.

## Weapons in Combat

The player can carry up to:

2 combat-ready weapons

These can be used without accessing the wider inventory.

Switching to a weapon outside the current combat-ready pair:

consumes the player's turn.

The same principle applies to armor changes.

Changing prepared equipment during battle has a real action cost.

## Armor Changes in Combat

The player may change armor during combat if the system permits it.

However:

Changing armor consumes a turn.

Do not make combat loadout changes free by default.

This preserves preparation as meaningful.

## Bonus Actions

Bonus Actions are a separate action category.

Certain:

- items
- effects
- conditions
- passives
- equipment
- Nodes

may grant Bonus Actions.

A Bonus Action may be performed:

at any moment during any turn

when its conditions permit.

This applies to the player AND enemies.

The Bonus Action system must therefore not be implemented as a player-only UI shortcut.

It is an engine-level combat mechanic.

## Bonus Action Interruptibility

Bonus Actions may occur during another actor's turn.

The engine must therefore support interrupt-like action insertion.

Conceptually:

```text
Normal Action resolving
↓
Bonus Action condition becomes valid
↓
Bonus Action inserted/resolved
↓
Original resolution continues
```

Exact nesting/recursion limitations must be controlled to prevent infinite trigger loops.

The engine should maintain explicit trigger/action depth safeguards.

## Fleeing

Fleeing is a legitimate combat action.

The exact success conditions are not yet defined.

Do not invent an arbitrary universal flee percentage.

The system should allow flee behavior to depend on factors such as:

- encounter
- enemy
- Priority
- effects
- environment
- special rules

## End-of-Round Processing

After all declared actions resolve, process end-of-round systems.

These may include:

- damage over time
- healing over time
- status effects
- resource regeneration
- environmental effects
- passive triggers

Then relevant:

- durations
- counters
- potencies
- temporary states

are reduced or updated.

Exact update behavior depends on each effect.

Do not assume every effect simply loses 1 duration.

## Combat Explainability

Priority and action ordering must be explainable.

A detailed combat/debug log should be capable of showing something similar to:

```text
Round 42

Declared Actions:

Player:
Arc Lightning

Bandit:
Quick Stab

Dragon:
Inferno Breath

Priority:

Bandit:
Dexterity +180
Haste +30
Action Weight -15
Final: 195

Player:
Dexterity +120
Spell Cast Weight -80
Equipment +10
Final: 50

Dragon:
Dexterity +40
Cast Time -70
Final: -30

Resolution Order:

1. Bandit — Quick Stab
2. Player — Arc Lightning
3. Dragon — Inferno Breath
```

Priority should not behave like an unexplained hidden dice roll.

## Unresolved Timing Details

The opening advantage is explicit, but the source leaves the exact opening action/round condition unspecified. Preserve the advantage without assuming an extra complete round or a specific queue algorithm. The round outline mentions durations generically; Status Effect lifetimes use Counter and effect-specific events as defined in [status effects](effects.md). Bonus Action nesting limits and flee success rules are also not numerically defined.
