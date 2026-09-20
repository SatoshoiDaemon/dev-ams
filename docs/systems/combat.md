# Combat

> **Specification status:** The [Canonical Quantified Combat Contract](#canonical-quantified-combat-contract) at the end of this document is normative. Earlier conceptual sections preserve design intent; where they conflict with the canonical contract, the canonical contract wins.

Related rules: [status effects](effects.md), [attributes and resources](attributes.md), [Scaling](scaling.md), [Fighting Styles](fighting-styles.md), and [equipment](equipment.md). The shared deterministic arithmetic and modifier structure is planned in [Numeric Design](../dev/numeric-design.md).

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

The player's controlled group receives the first action opportunity when combat begins. The player chooses which eligible controlled party member performs it.

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

Fleeing has no success roll or additional escape system in the base combat rules.

When the player selects `Flee`:

1. the current combat ends immediately;
2. the player receives no rewards from that combat, including experience, currency,
   items, or other combat-completion rewards;
3. no victory, kill, or combat-completion event is emitted;
4. the game returns to the applicable non-combat state.

Fleeing does not enter the damage pipeline, does not require a target, and does not
consume a normal combat action. The base game does not add encounter-specific escape
conditions, a universal flee percentage, a cost, a cooldown, or a failure chance.

The combat result must identify the outcome as `Fled` so that exploration state,
logs, and `explain last` can distinguish it from `Victory`, `Defeat`, and `Aborted`.

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

## Canonical Quantified Combat Contract

### Combat actors and turns

Every combat actor with active actions receives an independent turn. Passive-only entities do not receive turns. A turn begins with:

```text
AP = 100
```

AP does not accumulate. Unspent AP is discarded when the actor's turn ends. Equipment, attributes, or Priority do not increase the default AP maximum; a specific ability may explicitly modify AP.

Action costs are:

```text
Standard action = 100 AP
Quick action = 50 AP
Free action = 0 AP
Bonus Action = 50 AP unless its definition explicitly says otherwise
```

AP is an action budget, not initiative. Priority never grants additional actions.

An actor may reserve `50 AP` for a declared reaction such as Parry. Reserved AP cannot be spent on another action and is consumed only if the reaction is attempted.

### Action declaration and validation

An action is represented as an explicit object before resolution:

```text
ActionDeclaration:
    actor_id
    action_id
    source_id
    target_ids
    ap_cost
    cast
    required_resources
    declared_round
    action_priority
```

The initial TargetSpec validation occurs before AP and declared resources are paid. Once the action is accepted and paid, those costs are not refunded when a later resolution validation fails unless the action definition explicitly provides a refund.

Resolution performs a second validation for actor existence, incapacitation, equipment, target validity, and other action-specific requirements according to the canonical [Targeting](../dev/technical-contracts.md#targeting) contract. Combat targeting is selection-based, not spatial: the generic engine does not use distance, positioning, line of sight, line of effect, obstacles, or attack ranges. A failed validation produces a rejection record and no damage context.

### Cast and eligibility

`Cast` is a number of round boundaries, not merely a Priority penalty:

```text
Cast 0 = eligible during the declared round
Cast 1 = eligible during the next round
Cast 2 = eligible after two round boundaries
```

```text
eligible_round = declared_round + Cast
```

An action with `Cast > 0` enters the pending-cast queue and cannot resolve before its eligible round, regardless of Priority. Cast may optionally provide a secondary `-20 × Cast` action-priority modifier, but this is not required by the core contract.

Cast interruption rules are:

```text
Death or incapacitation → cancel Cast
Silence → cancel a magical Cast
Disarm → cancel a weapon-dependent Cast
Lost target → revalidate the target at resolution
Lost required state → fail the action at resolution
```

The canonical contract does not introduce a separate resource named `Magic Resistance`; magical defense uses the shared Defense system described below.

### Round and action queue

At the start of a round, all actors declare available actions. `Cast 0` actions enter the eligible queue; later Cast actions enter the pending queue. The eligible queue is ordered by Final Priority and then by deterministic tie-breakers:

```text
1. Higher Final Priority
2. Player party before non-party actors
3. Stable entity ID ascending
```

The first valid action of the player party is guaranteed to resolve before normal queue ordering once when combat begins. This does not grant the party a free round or permanent first Priority.

### Priority

```text
actor_priority = 100 + 2 × Dexterity

final_priority =
    actor_priority
    + action_priority
    + equipment_priority
    + active_effect_priority
```

Priority is signed and explainable. A higher value changes order only; it does not reduce Cast, increase AP, or grant extra actions. Priority modifiers may have limits only when their owning mechanic explicitly requires them. The core contract does not impose a Priority balance cap or create a universal random initiative roll.

### Attack and damage contexts

An attack is resolved in two contexts:

```text
AttackContext → determines whether the attack connects
DamageContext → calculates damage after a successful connection
```

An attack that misses or is successfully parried never creates a DamageContext.

### Accuracy and Evasion

Accuracy and Evasion use one opposed calculation rather than separate hit and evade rolls:

```text
accuracy_rating =
    floor(Precision / 5)
    + equipment_accuracy
    + active_effect_accuracy
    + ability_accuracy

evasion_rating =
    floor(Dexterity / 5)
    + equipment_evasion
    + active_effect_evasion
    + ability_evasion

effective_evasion = max(0, evasion_rating - accuracy_rating)

evasion_chance =
    effective_evasion / (effective_evasion + 200)
```

If `evasion_rating` is zero after modifiers, a valid attack cannot miss because of Evasion. Explicit action properties may mark an attack `Unavoidable`; this does not make it automatically Unparryable or Unblockable.

### Parry

Parry is a reaction supplied by a weapon, shield, Fighting Style, ability, or effect. It is not a universal passive defense. A Parry costs `50 AP` and requires reserved or otherwise available AP, a valid Parry source, and an attack eligible for Parry.

```text
effective_parry = max(0, defender_parry_rating - attacker_accuracy_rating)

parry_chance =
    clamp(effective_parry / (effective_parry + 150), 0%, 75%)
```

Successful Parry is binary: it negates the attack and emits `OnParry`. It does not reduce damage partially. `Unparryable` attacks skip this check without becoming Unavoidable or Unblockable.

### Shared Defense and damage-type modifiers

Physical and Magical damage use the same Defense contract. There is no separate core `Magic Resistance` attribute.

```text
effective_defense = max(
    0,
    Defense
    + conditional_defense[damage_type]
    - penetration[damage_type]
    - universal_penetration
)

defense_multiplier =
    floor(10_000 / (10_000 + effective_defense))
```

The multiplier is applied to positive damage after offensive modifiers and before Block. Physical and Magical damage may use different conditional Defense modifiers, equipment bonuses, effects, buffs, debuffs, or penetration sources, but they use the same mathematical curve. True Damage ignores Defense and penetration while still interacting with Shield and HP as defined by the damage pipeline.

### Block

Block is a deterministic reaction supplied by a shield, equipment, ability, or effect. It costs `50 AP` and requires a valid Block source. Block is evaluated after Defense mitigation and before Shield:

```text
blocked_damage = min(mitigated_damage, block_power)
remaining_damage = mitigated_damage - blocked_damage
```

`Block Power` is never negative. A successful Block emits `OnBlock`. The core contract does not define `OnBlockBreak`; that event requires a future persistent Block resource or guard meter.

An attack may independently be `Unblockable`. This property does not imply Unavoidable or Unparryable.

### Shield, Tenacity, and HP

Damage resolution follows:

```text
Base Damage + Scaling Contributions
→ offensive modifiers
→ shared Defense mitigation
→ Block
→ Shield
→ Tenacity
→ HP
```

Shield is a temporary damage layer. It never becomes negative. By default, Shields do not stack; a newly applied Shield replaces the current Shield only when its value is greater. A data definition may explicitly declare a Stackable Shield exception.

```text
shield_damage = min(remaining_damage, current_shield)
overflow = remaining_damage - shield_damage
```

While Tenacity is positive, applicable overflow is split `50%` to HP and `50%` to Tenacity. Odd points go to HP. If Tenacity cannot receive its share, the unreceived amount continues to HP. True Damage bypasses Tenacity but still passes through Shield and then HP.

### Combat events and death

The core event meanings are:

```text
OnEvade          → valid attack missed due to Evasion
OnParry          → valid attack was negated by Parry
OnBlock          → Block absorbed positive damage
OnAttackReceived → an attack effectively connected with the defender
OnDamageReceived → damage actually reduced at least one receiving damage layer
OnShieldDamage   → Shield decreased
OnTenacityDamage → Tenacity decreased
OnHPDamage       → HP decreased
OnHPBelow        → HP crossed a configured threshold
OnKill           → death was confirmed by Death Check
```

An attack fully negated by Evasion or Parry emits neither `OnAttackReceived` nor `OnDamageReceived`. Damage fully absorbed by Shield does not emit `OnDamageReceived`. A fully blocked attack emits `OnBlock` but does not emit damage-layer events. A Sigil-produced event is a new event and may activate eligible sources. `OnHPBelow` has no global once/while-below policy; the listener defines its behavior. Trigger processing occurs after resource resolution and before Death Check. Status-created actions are queued rather than recursively executed inline and remain subject to the global trigger-depth and event-count safeguards.

### Scaling contribution

Scaling uses the shared rule in [Attribute Scaling](scaling.md):

```text
scaling_contribution = floor(attribute_power × scaling_multiplier)

pre_mitigation_damage =
    flat_base_damage + sum(scaling_contributions)
```

Power is not damage by itself. Each weapon, spell, catalyst, or ability must declare its Scaling sources and grades.

### Explain Last requirements

For every resolved or rejected action, detailed logs should preserve:

```text
action declaration and validation
AP and resource payment
Cast and eligible round
Priority contributions and queue position
selected targets, target relations, and invalid targets removed at resolution
Accuracy and Evasion inputs/result
Parry or Block inputs/result
Scaling contributions
Defense and penetration inputs
Shield, Tenacity, and HP before/after
events emitted
trigger depth and rejection reason, if any
```
