# Magic Nodes and Glyphs

> **Specification status:** The section [Canonical Quantified Specification](#canonical-quantified-specification) is normative. The older catalog below preserves the original design intent and vocabulary; where it conflicts with the quantified section, the quantified section wins. All values use the shared integer/floor rules from [Numeric Design](../dev/numeric-design.md).

This document preserves the initial magic Node and Glyph design proposals and now adds the implementation contract for them. Equipment-specific behavior is documented in [Equipment Sigils](sigils.md). The authoritative rules for Counter, Potency, status competition, status events, and individual effects remain in [Status Effects](effects.md).

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

## Historical Open Design Details

The following bullets describe gaps that existed before the canonical quantified specification. They are retained for provenance only; the section below resolves them.

These proposals preserve the original examples without defining missing mechanics:

- The status budget, Weight calculation, conversion rates, and Glyph efficiencies are not specified. For example, the preserved `20/20 → 35/8` conversion does not establish a rule that Counter plus Potency must remain numerically constant. The displayed Weights `18` and `31` are illustrative.
- Turn, action, decay, and trigger timing must be integrated with the canonical combat and status systems. The Node grammar's proposed trigger labels do not define new event API contracts by themselves.
- Elemental Glyphs intentionally propose changes to normal status behavior; their exact scope, limits, and interaction with competition remain to be defined. No unspecified stacking, direct-HP bypass, or additional status property is implied beyond each stated proposal.
- The Focus example places `Fire` under `Damage`, while the general grammar describes an Elemental Type branch. Both source examples are preserved; their exact graph representation needs a single consistent schema.

## Canonical Quantified Specification

This section closes the numerical gaps above. It is deliberately conservative: a Glyph changes an existing operation and never creates an unbounded resource loop.

### Shared spell and editor rules

The editor uses the following integer formula:

```text
Spell Weight = max(1, sum(Node Weight + parameter Weight + Glyph Weight) - sum(Sacrifice Credit))
Spell Capacity = 20 + 2 × Intelligence + 5 × Catalyst Tier + 3 × Grimoire Rank
```

`Intelligence`, `Catalyst Tier`, and `Grimoire Rank` are non-negative integers. A spell is valid only when `Spell Weight <= Spell Capacity`; a cast also requires its Mana cost. Node weights are: Source `0`, Type `1`, Form `2`, Effect `2`, Modifier `1`, Sacrifice `0`. Parameter buckets cost `1 + floor((value - 1) / 10)` for positive integer values. A Glyph's listed Weight is added once per spell instance. Sacrifice Credits are `Cast Time +1`, `Cooldown +1`, `Cost +1`, `HP Cost`, `Self Status`, `Condition`, `Tribute`, and `Magic Slots +`; their credits are respectively `2, 2, 2, 3, 2, 2, 3, 4` and may not reduce a spell below Weight `1`.

Base Mana Cost is `5 + Spell Weight`; cast time is `1 action`; cooldown is `0` unless a Glyph below adds one. Every percentage is calculated in basis points and floored at the stated step. A source event can activate one instance of a Glyph once, one spell can affect at most 8 targets, and a spell can emit at most 3 secondary effects. A Glyph requiring an element is invalid unless that element is present in the same branch. Elemental ownership, node eligibility, and status competition are checked before activation.

Status applications use `Status Budget = Potency × potency_rate + Counter × counter_rate`. Default rates are Burn `4/5`, Poison `1/1`, Bleed `2/1`, Electrified `2/1`, Tremor `1/2`, and all other statuses `1/1`. The default cap is `100` budget, Potency `100`, Counter `100`; crowd control has Potency `1` and Counter `1..5`. A transformation may not exceed these caps. Unless a row says otherwise, Glyph output snapshots values at activation, rounds down, and cannot activate recursively from its own secondary effect.

### General status Glyph cards

| Glyph | Weight | Exact rule | Requirement and limit |
|---|---:|---|---|
| Condensed | 4 | Burn: convert budget to `Potency += floor(Counter × 3/5)`, then set `Counter = max(1, floor(Counter × 2/5))`; other statuses use `Potency += floor(Counter/2)`, `Counter = ceil(Counter/2)`. | Apply Status; once per application; budget may not increase. |
| Lingering | 4 | Burn: `Counter += floor(Potency × 3/5)`, `Potency = max(1, floor(Potency × 2/5))`; other statuses use `Counter += floor(Potency/2)`, `Potency = ceil(Potency/2)`. | Apply Status; once per application. |
| Stable | 5 | Prevents the first `2` natural Potency-decay events; Counter still decays normally. | Burn, Bleed, Poison, Rupture only; one Stable per status. |
| Accelerated | 4 | Executes one status tick immediately and consumes `2 Counter`; if Counter < 2, invalid. | Tick-capable status; once on application. |
| Delayed | 3 | Delay is exactly `2 turns`; on release, Potency is `+25%` (floor), Counter unchanged. The queued status is cancelled if its source is removed. | One delayed instance per target; no stacking. |
| Contagious | 6 | On target death, transfers `50%` of remaining Potency and `50%` of remaining Counter to the nearest valid target within `4` tiles. | One transfer, max one target, no transfer from crowd control. |
| Embedded | 6 | On cleanse/removal, deals `25%` of remaining status budget as True Damage, capped at `20`; does not trigger on expiry. | Debuff only; once per instance. |
| Threshold | 2 | Applies only when target HP is `<=50%` or target has `>=3` debuffs. When valid, Glyph Weight is `-1` (minimum 1); when invalid, no status is created. | One chosen condition, fixed at edit time. |

### Elemental Glyph cards

All rows require the named element in the branch. `W` is Glyph Weight. “Once” means once per source event; “spell” means once per cast.

#### Fire

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Ignition | 5 | Consume `1..5` Burn Counter (chosen at edit time) and deal `Potency × consumed Counter / max(1, Counter)` Magical Fire Damage. | Once per target per hit; requires Burn. |
| Kindling | 4 | Store `50%` of Potency lost by Burn, max `20`; next Burn by caster gains stored Potency and clears store. | One store per caster; expires after 3 turns. |
| Flashover | 6 | Copy `30%` of highest Burn Potency to each other hit target, Counter `50%` of source Counter. | Max 3 secondary targets; no chain copy. |
| Cremation | 7 | On Burned target death, explode for `50%` of remaining Burn budget as Magical Fire Damage. | Max `40` damage; once per death. |
| White Flame | 4 | Normal Fire Damage `-20%`; damage to Shield `+50%`. | Applies to the whole spell. |

#### Water

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Flow | 3 | The final Chain may return to one previously hit target at `50%` of original effect. | One return; no return if target is dead. |
| Dilution | 4 | Sacrifice `20%` of the new status Potency to remove `20%` of another selected debuff Potency. | One selected debuff; no budget creation. |
| Confluence | 4 | Convert `50%` of overheal into Shield, capped at `25%` of caster Max HP. | Heal effect only; once per cast. |
| Undertow | 3 | Water Pull also applies `Priority -10` to the target's next action. | One action; cannot stack. |
| Equalize | 6 | In Area, replace selected status Potencies with their floor average; Counter remains the minimum of affected instances. | Max 4 targets; total Potency is conserved within floor loss. |

#### Earth

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Foundation | 4 | If caster moved `0` tiles since last action, spell output and Shield are `+20%`. | Snapshot at cast; breaks after movement. |
| Seismic | 4 | Each prevented Push tile deals `5` Tenacity Damage, max `25`. | Push effect only; once per target. |
| Faultline | 6 | Copy `30%` of Tremor Potency and Counter to up to 2 other Area targets. | Requires Tremor; once per cast. |
| Bedrock | 5 | Shield created is `-25%` HP but gains `+50% Resistance Scaling` when calculating Shield strength. | Shield effect only. |
| Erosion | 5 | Each consecutive Earth hit on same target grants `+10%` Tenacity Damage, max `+50%`; changing target resets. | One sequence per caster. |

#### Air

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Tailwind | 3 | For every `10` Priority caster advantage, Push and spell speed gain `+5%`, max `+25%`. | Snapshot at cast. |
| Slipstream | 3 | Successful Push, Pull, or Teleport grants caster `+15 Priority` on next action. | Once per cast; does not stack. |
| Ricochet | 6 | A missed/evaded Projectile retargets one valid target within `4` tiles at `60%` effect. | One ricochet; second miss ends spell. |
| Vacuum | 4 | Pulls one Projectile or Trap within `3` tiles by up to `2` tiles toward the chosen point. | Max 2 objects per cast. |
| Updraft | 5 | Damage `-50%`; Push distance `×2`; interrupts channeling if Push is `>=2` tiles. | One target; no damage increase from other modifiers. |

#### Light

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Purification | 4 | Spend `25%` of Heal amount to remove `25%` of one debuff's remaining budget. | Max 2 debuffs per cast; floor both values. |
| Revelation | 4 | Removes Invisible and applies `Reveal Lock` for `3 Counter`; Invisible cannot reapply during it. | One target per hit. |
| Judgment | 5 | Consume up to `20` enemy Buff Potency; gain `+1%` spell damage per 2 consumed, max `+10%`. | One buff; ends after spell. |
| Sanctuary | 6 | Positive Aura effects are shared at `50%` strength with up to 3 allies in Area. | Allies only; one Aura. |
| Absolution | 4 | Fully cleansing a debuff restores `5%` caster Max HP and `2%` Max Mana per 20 removed budget, capped at `10%/5%`. | Once per target per cast. |

#### Darkness

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Devour | 5 | Consume up to `10` debuff Counter; each consumed Counter adds `2%` to Drain, max `+20%`. | One target; consumed status cannot tick that Counter. |
| Corruption | 5 | Removing an enemy Buff converts `50%` of removed Potency into Curse Potency, max `20`. | One Buff; Curse competition applies. |
| Shadow Debt | 4 | Reduce current spell Weight and Mana Cost by `3`; after `3 turns`, pay `20 HP` and `10 Mana`, or take `10 True Damage` per unpaid resource. | One debt; cannot reduce Weight below 1. |
| Usurpation | 7 | On kill, copy one Buff at `50%` Potency/Counter to caster for `3 turns`. | One Buff, one kill, no crowd control. |
| Eclipse | 4 | Against a target with no ally within `3` tiles, effect is `+25%`; at 1/2/3+ allies, bonus is `15%/5%/0%`. | Snapshot at activation. |

#### Poison

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Toxic Catalyst | 5 | Each distinct Debuff on target increases Poison tick Potency by `5%`, max `+50%`. | Counts up to 10 debuffs; snapshot per tick. |
| Necrosis | 6 | `20%` of Poison tick damage becomes direct HP damage; Poison Potency is `-30%`. | Direct HP portion ignores Shield and Tenacity; max 10 per tick. |
| Anticoagulant | 4 | When Bleed activates, advance Poison by `1` tick and consume `1 Counter` from Poison. | Once per Bleed event. |
| Reservoir | 5 | On Poison replacement by higher Potency, preserve `50%` of old Counter, capped at `20`, added to new Counter. | Poison only; once per replacement. |
| Compound | 4 | One selected Debuff loses `25%` less Counter per decay event while Poison remains. | One Debuff; minimum decay is 1 when it normally decays. |

#### Ice

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Permafrost | 4 | The applied status skips its first natural decay event. | One status per application. |
| Brittle | 5 | Against crowd control, consume `1 Counter` to add `10` Tenacity Damage to the hit. | Max 3 activations per status. |
| Cold Snap | 5 | Consume up to `3 Counter`; deal `5 Magical Ice Damage` per Counter and reduce Potency by `1` per Counter. | One status per hit. |
| Cryostasis | 6 | Freeze Potency and Counter for exactly `2 turns`; frozen status cannot decay or be transformed. | One status; cannot target self. |
| Shatter | 7 | Breaking Shield or Tenacity adds `25%` of the value broken by the final hit as Ice Damage, max `30`. | Once per target per cast. |

#### Sound

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Echo | 8 | Repeat the last Effect after `2 actions` at `50%` output. | One repeat; queued effect is removed if source dies. |
| Resonance | 5 | Consecutive hits by same spell gain `+10%` effect per prior hit, max `+30%`; another spell resets. | Per caster-target pair. |
| Dissonance | 4 | Against a channel, interruption power is `×2`; successful interruption deals `10` Sound Damage. | One channel per cast. |
| Harmonic | 4 | Area effect gains `+5%` per target hit, max `+25%`. | Snapshot after target selection. |
| Feedback | 6 | If interrupted, `50%` of spent Mana and `50%` of primary effect hits interrupter. | Once; max `25` damage. |

#### Metal

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Magnetism | 3 | Push/Pull strength gains `+10%` per 100 equipment Weight on target, max `+50%`. | Equipment Weight is snapshotted. |
| Sharpen | 4 | Physical damage `-15%`; Armor Penetration `+25 percentage points`. | Physical Damage only. |
| Temper | 4 | Shield/Protect Counter is multiplied by `1 + armor_weight/200`, capped at `×2`. | Target must be equipped. |
| Shrapnel | 6 | A Pierce that passes through a target creates one secondary projectile at `40%` effect. | Max 3 secondary projectiles per cast. |
| Forge Bond | 5 | Weapon spell takes `30%` of its Scaling Power from equipped weapon Scaling and `70%` from magical Scaling. | Weapon Form; requires equipped weapon. |

#### Plant

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Growth | 5 | Persistent effect gains `+10%` output per active turn, max `+50%`; resets on reapplication. | Persistent effects only. |
| Root | 4 | Each prevented Push/Pull tile adds `1 Binding Counter`, max `5` per event. | Requires Binding; once per movement attempt. |
| Germination | 6 | An untriggered Trap leaves a child Trap at `50%` Potency and `50%` Counter. | One generation; max 2 children per cast. |
| Photosynthesis | 4 | If Plant Heal/Shield survives until next turn, restore `5%` Max Mana, max `10` Mana per cast. | Once per persistent effect. |
| Propagation | 6 | On affected target death, transfer `30%` status Potency/Counter to nearest target within `3` tiles or create a Trap at death location. | One transfer or Trap; max 2 activations per cast. |

#### Lava

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Molten Ground | 6 | Leaves a 2-tile Area for `3 turns`; entering or ending turn there receives `50%` original effect once per turn. | Max one terrain per cast; terrain does not stack. |
| Obsidian | 5 | On expiry, leaves a 1-tile Trap with `40%` original effect and `2 Counter`. | One Trap; max 2 concurrent Obsidian Traps. |
| Melt Armor | 5 | Consecutive Lava hits reduce physical Defense by `5%` per hit, max `20%`, for `3 Counter`; target switch resets. | One mark per target. |
| Viscosity | 4 | Push distance `-50%`; target loses `10 Priority` and `1 Movement` for `2 turns`. | Once per target per cast. |
| Eruption | 7 | Detonate terrain/Trap, consuming all remaining Counter; damage is `5 × Counter`, max `50`. | Manual action; one detonation per object. |

#### Lightning

| Glyph | W | Exact effect | Limit |
|---|---:|---|---|
| Conductor | 5 | Chain-connected targets form one network for `3 turns`; maximum 6 members. | New members require a Chain hit. |
| Overload | 5 | Consume up to `3 Electrified Counter`; deal `8 Lightning Damage` per Counter. | Once per target per hit. |
| Arc Jump | 4 | Chain prioritizes targets without Electrified; if none exist, next jump is `-25%` effect. | Max 5 jumps. |
| Grounding | 4 | With exactly one valid enemy, Electrified duration is `+2 Counter` and damage is `-20%`. | Snapshot at cast. |
| Impulse | 4 | If caster acts before target, Chain jumps and Electrified application are `+20%` effective. | Priority comparison at cast. |

### Catalyst specializations

Staff Focus costs `5 Weight`, selects exactly one Effect Node, and grants its attached Modifiers `+25%` efficiency (cap `+50%`); it cannot select a Source, Type, or Form. Grimoire Inscription costs `4 Weight`, stores up to `3` phrases, and reduces each repeated phrase's repertoire cost by `50%` but never reduces Mana. Seal Imposition costs `5 Weight`, changes one status budget exchange rate by `+25%` efficiency, capped by the normal status budget cap. Catalyst Tier is `0..5`; each tier adds `5` Spell Capacity and is required by the corresponding catalyst item.

### Worked examples and implementation tests

`Burn 25/12` has base status Weight `5 + bucket(25,3) + bucket(12,3) = 18`. Adding Condensed, Accelerated, and Ignition gives `18 + 4 + 4 + 5 = 31`, reproducing the original editor examples. `Burn 20/20` has budget `180`; Condensed yields `35/8`, budget `180`. `Poison 30/10` has budget `40`; Lingering yields `15/25`, budget `40`. A `Spell Weight 34` spell with Intelligence `7`, Tier `1`, Rank `0` has capacity `20 + 14 + 5 = 39` and is valid; the same spell with Intelligence `2` has capacity `29` and is rejected before casting. A Fire Ignition with Burn Counter `1` consumes one Counter; a second Ignition in the same hit is rejected by the per-target activation limit.

Implementations must test: every row's Weight and requirement; all caps; floor behavior; no-budget-creation for Condensed/Lingering; elemental branch rejection; target/activation limits; deterministic target ordering; and `explain last` fields for input snapshot, consumed values, produced values, Weight delta, and rejection reason.
