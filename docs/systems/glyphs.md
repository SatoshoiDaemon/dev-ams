# Magic Nodes and Glyphs

This document preserves the initial magic Node and Glyph design proposals. It defines a vocabulary and a catalog of proposed behaviors; illustrative values are not complete balancing formulas. Equipment-specific behavior is documented in [Equipment Sigils](sigils.md). The authoritative rules for Counter, Potency, status competition, status events, and individual effects are in [Status Effects](effects.md). The first candidate Weight, Status Budget, and numeric-card structure is documented in [Glyph Numeric Planning](../dev/glyph-balance.md).

## General Magic Node Grammar

The proposed grammar is:

```text
SOURCE
├─ Player Cast
├─ By Companion
├─ Passive
└─ Trigger
   ├─ Player Hit [threshold]
   ├─ Companion Hit [threshold]
   ├─ Player Damage [threshold]
   ├─ Companion Damage [threshold]
   ├─ Player Turn Start
   ├─ Player Turn End
   ├─ Enemy Turn Start
   ├─ Enemy Turn End
   └─ Status Event [status]

TYPE
├─ Elemental [element]
├─ Sacred
├─ Profane
└─ Pure Magic

FORM
├─ Projectile
├─ Ray
├─ Area
├─ Self
├─ Touch
├─ Weapon
├─ Trap
├─ Aura
└─ Summon

EFFECT
├─ Damage
├─ Apply Status
├─ Heal
├─ Push
├─ Pull
├─ Teleport
├─ Drain
├─ Revive
└─ Summon Creature
   ├─ Golem
   ├─ Undead
   └─ Imp

MODIFIER
├─ Damage+
├─ Area+
├─ Duration+
├─ Speed+
├─ Projectiles+
├─ Pierce
├─ Chain
├─ Repeat
├─ Homing
├─ Quantity
├─ Cost-
├─ Cooldown-
└─ Invisible

SACRIFICE
├─ Cast Time+
├─ Cooldown+
├─ Condition
├─ Cost+
├─ HP Cost
├─ Self Status
├─ Tribute
└─ Magic Slots+
```

`Duration+` is retained as a proposed Node label. For a Status Effect, remaining lifetime is its **Counter**; the label does not introduce an independent generic `Duration` property.

## General Status Glyphs

General magic Glyphs are independent of element. A status Node exposes Counter and Potency directly:

```text
Apply Status
└─ Burn

Potency: 20
Counter: 8
```

Weight accounts for both values. Consequently, `Burn 5/20` does not cost the same as `Burn 80/80`.

| Glyph | Proposed behavior |
| --- | --- |
| Condensed | Transfers part of Counter into Potency: the same status budget becomes more intense and shorter. |
| Lingering | Converts Potency into Counter, reversing the direction of Condensed. |
| Stable | Prevents natural Potency loss for part of the lifetime, but increases Weight. This is particularly relevant to Burn, Bleed, and Rupture, which normally lose Potency. |
| Accelerated | Executes the status effect immediately on application, but consumes additional Counter. |
| Delayed | Delays the start by X turns, then begins with higher Potency. |
| Contagious | When a target dies with the status, transfers part of it to another nearby valid target. |
| Embedded | Removing or cleansing the status also causes HP/Tenacity loss or another defined effect, so dispelling it need not be free. |
| Threshold | Applies the status only when a specified condition is satisfied, in exchange for better efficiency/Weight. |

The original conversion examples are:

```text
Burn 20/20
→ Condensed
→ Burn 35/8

Poison 30/10
→ Lingering
→ Poison 15/25
```

The player should be able to build the following and understand the result:

```text
Apply Status
└─ Poison
   ├─ Potency: 40
   ├─ Counter: 20
   ├─ Lingering
   └─ Contagious
```

For crowd control with fixed Potency, the editor locks the value:

```text
Stun
Potency: 1 [LOCKED]
Counter: 1–5
```

Incompatible Glyphs are unavailable, with an explanation of the incompatibility.

## Element-Exclusive Glyphs

### Fire — Combustion and Consumption

Fire converts accumulated states into immediate damage. Its intended loop is to apply, feed, and consume Burn.

| Glyph | Proposed behavior |
| --- | --- |
| Ignition | A Fire hit against a target with Burn consumes part of Burn's Counter to trigger its damage immediately. |
| Kindling | Stores part of the Potency that Burn loses naturally; the next Burn applied by the caster receives that Potency. |
| Flashover | When hitting multiple targets, replicates part of the Burn from the target with the highest Potency onto the others. |
| Cremation | If a target dies with Burn, consumes the remaining Burn and converts it into an explosion proportional to what remained. |
| White Flame | Fire deals less normal damage, but gains exceptional efficiency against Shield. |

### Water — Flow and Redistribution

Water manipulates existing effects and changes their destination.

| Glyph | Proposed behavior |
| --- | --- |
| Flow | Chain may return once to a previously hit target, with reduced efficiency. |
| Dilution | When applying a status, may sacrifice some of its Potency to reduce the Potency of another status on the target. |
| Confluence | Partially converts healing beyond maximum HP into Shield. |
| Undertow | Pull with Water also reduces the Priority of the next action. |
| Equalize | In an Area, redistributes a chosen status between targets, bringing their Potencies closer together without creating new Potency. |

An Equalize example: `10 / 10 / 70` could become approximately `30 / 30 / 30`.

### Earth — Stability, Tenacity, and Impact

Earth rewards holding position and makes its impacts threatening. It is the natural element for breaking Tenacity.

| Glyph | Proposed behavior |
| --- | --- |
| Foundation | If the caster has not changed position since their last action, the spell gains additional efficiency. |
| Seismic | Push that cannot move its target converts the unapplied distance into Tenacity damage. |
| Faultline | Hitting a target with Tremor may spread part of that Tremor to other targets in the Area. |
| Bedrock | Shield created by the branch receives less HP, but benefits from Resistance. |
| Erosion | Successive Earth hits against the same target progressively reduce its resistance to Tenacity damage; switching targets resets this progression. |

### Air — Movement, Priority, and Trajectory

Air is distinct from Lightning: it manipulates where, when, and how things move. Its role is positioning and timing.

| Glyph | Proposed behavior |
| --- | --- |
| Tailwind | A greater caster Priority advantage over the target increases the spell's speed/Push. |
| Slipstream | Successful Push, Pull, or Teleport grants Priority to the caster's next action. |
| Ricochet | A Projectile that misses or is evaded may change trajectory toward another valid target. |
| Vacuum | Pull may affect Projectiles/Traps present in the Area, repositioning them. |
| Updraft | Sacrifices damage to greatly increase Push and interrupt certain channeled actions. |

### Light — Purification, Revelation, and Protection

Light removes harmful effects and punishes an opponent's artificial advantages.

| Glyph | Proposed behavior |
| --- | --- |
| Purification | Heal may spend part of its healing to remove Counter/Potency from debuffs. |
| Revelation | A hit removes Invisible and prevents its reapplication for a specified Counter. |
| Judgment | May consume Potency from enemy Buffs to increase the spell's offensive effect. |
| Sanctuary | Positive Aura effects may be partially shared among allies inside it. |
| Absolution | Fully removing a debuff from an ally restores a small amount of HP/Mana proportional to the removed debuff. |

### Darkness — Appropriation, Corruption, and Debt

Darkness purchases immediate power with resources belonging to the future or to the enemy.

| Glyph | Proposed behavior |
| --- | --- |
| Devour | Drain may consume Counter from the target's debuffs to increase the resource drained. |
| Corruption | Removing an enemy Buff converts part of its Potency into Curse. |
| Shadow Debt | Immediately reduces Weight/Mana Cost, but accumulates an HP/Mana debt collected later. |
| Usurpation | Killing a target allows a Buff it possessed to be temporarily stolen. |
| Eclipse | Gains efficiency against isolated targets; this advantage decreases as the number of nearby allies increases. |

### Poison — Deterioration and Debuff Synergy

Poison should have a broader identity than simply applying the Poison status. It rewards builds that apply many different debuffs—the original design example was to put "14 terrible things on this target."

| Glyph | Proposed behavior |
| --- | --- |
| Toxic Catalyst | Each different Debuff on the target accelerates Poison progression. |
| Necrosis | Part of Poison damage goes directly to HP, but its effective Potency decreases. |
| Anticoagulant | When Bleed activates, also advances one instance of Poison. |
| Reservoir | Replacing Poison with a higher-Potency instance preserves part of the previous Counter. |
| Compound | Select another Debuff in the spell; while Poison is present, that Debuff's Counter decays more slowly. |

### Ice — Preservation and Breaking

Ice freezes states in time and then destroys them. Water redistributes; Ice preserves and breaks. Ice's identity is more than defensive Water.

| Glyph | Proposed behavior |
| --- | --- |
| Permafrost | A newly applied status skips its first natural decay. |
| Brittle | Attacks against a target under crowd control deal more Tenacity Damage by consuming Counter from the crowd control effect. |
| Cold Snap | Consumes part of a status's Counter to produce immediate damage proportional to the amount removed. |
| Cryostasis | Select a Buff/Debuff; it temporarily neither gains nor loses Potency/Counter. |
| Shatter | Breaking Shield or Tenacity with Ice produces additional damage based on the value destroyed by the final hit. |

### Sound — Resonance, Repetition, and Interruption

Sound can be one of the more mechanically unusual elements. It suits Repeat, Aura, Area, and anti-caster builds while retaining an identity distinct from Lightning.

| Glyph | Proposed behavior |
| --- | --- |
| Echo | Part of the last Effect executed by the spell occurs again after X actions, with reduced efficiency. |
| Resonance | Repeatedly hitting the same target with the same spell progressively increases its effect; using another spell breaks the sequence. |
| Dissonance | Against an enemy channeling an action, Sound gains exceptional interruption capability. |
| Harmonic | An Area hit increases the effect according to the number of targets hit. |
| Feedback | If a Sound spell is interrupted, part of its effect immediately returns against whoever caused the interruption. |

### Metal — Armor, Penetration, and Equipment

Earth controls Tenacity; Metal interacts directly with equipment. Metal is the natural element for weapon magic.

| Glyph | Proposed behavior |
| --- | --- |
| Magnetism | Push/Pull gains efficiency based on the target's weight/metal equipment. |
| Sharpen | Physical Damage produced by the spell gains Armor Penetration but loses part of its raw damage. |
| Temper | Shield/Protect applied to an equipped target lasts longer with heavier armor. |
| Shrapnel | Pierce that passes through a target generates secondary projectiles with part of the original effect. |
| Forge Bond | A Weapon + Metal spell may partially use the equipped weapon's Scaling instead of depending exclusively on magical Scaling. |

Forge Bond is particularly relevant to a Battlemage:

```text
Player Cast
└─ Elemental: Metal
   └─ Weapon
      └─ Damage
         └─ Forge Bond
```

### Plant — Growth and Persistence

Plant is an investment that grows while it remains alive. It works especially well with Trap, Aura, Heal, Binding, and long lifetimes.

| Glyph | Proposed behavior |
| --- | --- |
| Growth | Persistent effects become progressively stronger each turn they remain active. |
| Root | Push/Pull prevented by Binding is converted into additional Binding Counter. |
| Germination | A Trap that expires without activating leaves a smaller version of itself at the location. |
| Photosynthesis | Heal/Shield applied by Plant recovers a small amount of Mana if it remains active until the next turn. |
| Propagation | When an affected target dies, part of a Plant status may pass to a nearby unit or create a Trap at the location. |

### Lava — Territory and Inevitability

Fire works with Burn; Lava works with dangerous areas and slow, heavy, brutal effects. Its identity should extend beyond higher-damage Fire. Lava provides territorial control that can be converted into burst damage.

| Glyph | Proposed behavior |
| --- | --- |
| Molten Ground | Area leaves persistent terrain that reapplies part of the effect to anyone remaining in or entering it. |
| Obsidian | When a Lava effect expires, it may leave a solid defense/Trap at the location. |
| Melt Armor | Consecutive Lava hits temporarily reduce physical Defense/Resistance. |
| Viscosity | Weakens Push, but enemies hit lose Movement/Priority proportionally. |
| Eruption | A Lava Trap/Area may be detonated manually, consuming its remaining lifetime for a single effect proportional to the remaining Counter. |

Eruption example:

```text
Molten Ground
Remaining Counter: 17

ERUPTION
→ sacrifices the next 17 turns
→ BOOM
```

### Lightning — Networks, Speed, and Overload

Lightning is fast, spreads, and rewards building a network.

| Glyph | Proposed behavior |
| --- | --- |
| Conductor | Targets connected by Chain count as members of the same Electrified network. |
| Overload | Hitting an Electrified target again consumes Counter for immediate damage. |
| Arc Jump | Chain prioritizes enemies that do not yet have Electrified. |
| Grounding | When there is only one relevant enemy, Electrified changes its behavior so it does not depend on a crowd. |
| Impulse | Resolving the spell before the target through Priority improves its Electrified/Chain application. |

## Catalyst Glyph Specializations

Catalysts need their own space for Glyph behavior. Keep **Catalyst Property**, **Core**, and **Glyph** conceptually separate.

### Staff — Offensive Structure

**Focus** is the proposed exclusive Sigil/Glyph: select one Effect Node in the spell. Modifiers attached to that Effect gain better efficiency, while the other modifiers retain their normal efficiency.

Example favored by this design:

```text
Projectile
└─ Damage ← FOCUSED
   ├─ Fire
   ├─ Damage+
   └─ Pierce
```

### Grimoire — Complexity and Repertoire

**Inscription** stores a small subtree as a reusable "phrase." Repeating it in different spells uses less repertoire capacity. It improves storage and does not necessarily reduce Mana cost.

### Seal — Statuses and Conditions

**Imposition** redistributes the budget between Potency and Counter more efficiently than other catalysts.

```text
Normal catalyst:
40 Status Budget
→ 20 Potency / 20 Counter

Seal:
40 Status Budget
→ allows more extreme specializations
→ 35 / 8
or
→ 10 / 38
```

The Core adds a separate layer:

```text
CATALYST
│
├─ Type: Staff
│   └─ Focus mechanic
│
├─ Core: Dragon Heart
│   ├─ favors Fire
│   ├─ favors Area
│   └─ interacts with Burn
│
└─ Sigils
    └─ other modifications
```

## Universal Status Interface

Every Apply Status should explicitly expose its parameters, Weight, and natural behavior:

```text
STATUS: Burn

Potency: [ 25 ]
Counter: [ 12 ]

Final Weight: 18

Natural behavior:
- Deals Potency Magical Damage
- -1 Potency/turn
- -1 Counter/turn
```

After adding Glyphs, the interface example becomes:

```text
STATUS: Burn

Potency: 25
Counter: 12

Glyphs:
[Condensed]
[Accelerated]
[Ignition — FIRE]

Final Weight: 31
```

An elemental Glyph works only when its subtree has the required Elemental Type. If Cremation requires Fire affinity in the branch, the following is invalid:

```text
Pure Magic
└─ Apply Burn
   └─ Fire Glyph: Cremation
```

This is valid:

```text
Elemental: Fire
└─ Apply Burn
   └─ Cremation
```

The graph's topology therefore affects which behaviors are available.

## Node, Glyph, and Sigil Responsibilities

A Glyph should answer at least one of these questions:

- When does something happen differently?
- How does it interact with another mechanic?
- What happens to something that would normally be wasted?
- Which normal rule does it change?

If its only possible description is "+15% damage," it probably belongs in a Modifier Node, material, Scaling, or attribute.

```text
NODE
"What IS this thing, or what does it DO?"

GLYPH
"How does this thing behave in a special way?"

SIGIL
"How does a piece of equipment behave in a special way?"
```

## Open Design Details

These proposals preserve the original examples without defining missing mechanics:

- The status budget, Weight calculation, conversion rates, and Glyph efficiencies are not specified. For example, the preserved `20/20 → 35/8` conversion does not establish a rule that Counter plus Potency must remain numerically constant. The displayed Weights `18` and `31` are illustrative.
- Turn, action, decay, and trigger timing must be integrated with the canonical combat and status systems. The Node grammar's proposed trigger labels do not define new event API contracts by themselves.
- Elemental Glyphs intentionally propose changes to normal status behavior; their exact scope, limits, and interaction with competition remain to be defined. No unspecified stacking, direct-HP bypass, or additional status property is implied beyond each stated proposal.
- The Focus example places `Fire` under `Damage`, while the general grammar describes an Elemental Type branch. Both source examples are preserved; their exact graph representation needs a single consistent schema.
