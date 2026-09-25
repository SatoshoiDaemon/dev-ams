# Character Creation, Races, and Element Ownership

The provisional narrative background is documented in [world lore](../world/lore.md). Further elemental progression is defined in [Elemental Trials](elemental-trials.md).

## Character Creation

At the beginning of the game, the player creates their character.

The two primary initial decisions are:

```text
Race
+
Initial Elements
```

These decisions should affect gameplay immediately.

## Races

Initial playable races:

- Human
- Demon
- Angel
- Elf
- Feral
- Dragonborn

Human, Demon, and Angel have the canonical initial racial kits defined below. Elf, Feral, and Dragonborn remain playable-race declarations whose mechanics require a later explicit design decision; the engine must not invent placeholder bonuses for them.

Race differences are mechanical, data-driven, and available to mods. Stable base-game race IDs are:

```text
base:human
base:demon
base:angel
base:elf
base:feral
base:dragonborn
```

The initial ability IDs are `base:natural-experience`, `base:hard-work`, `base:talented`, `base:dread-of-society`, `base:broken-heart`, `base:purgatory-fire`, `base:sanctity`, `base:halo`, and `base:purge` in the same order as the kits below.

Racial percentages use basis points and follow the global floor-rounding rule. Racial definitions, ability values, granted Nodes, and granted spell templates belong in the content registry rather than race-specific `match` branches. A save stores the stable race ID and any persistent racial choices.

## Human

### Passive — Experiência Nata

Humans receive `+10%` experience from every positive experience award.

```text
Human Experience = floor(Base Experience × 1.10)
```

The modifier applies once to the award received by the Human. It does not increase a companion's separate award and does not turn a zero or negative value into positive experience.

### Passive — Trabalho Duro

Whenever a Human gains a level, they receive `+2` distributable attribute points in addition to the normal progression award.

The normal award remains one point each level and one additional point every fifth level. Therefore a Human receives:

```text
Ordinary level: 1 + 2 = 3 attribute points
Every fifth level: 1 + 1 + 2 = 4 attribute points
```

This passive does not grant points retroactively when a character changes race through a mod or save edit. A migration or race-changing effect must declare its own point policy rather than silently duplicating past awards.

### Active — Talentoso

At character creation, a Human may construct one unique active ability from legal Nodes and Glyphs available to that character. The player may edit the graph until confirming it. Once confirmed, its graph, parameters, and stable component IDs are permanent for that save and cannot be changed by ordinary respec.

Talentoso has `50%` more Node/Glyph capacity than an ordinary structure built from the same base capacity:

```text
Talent Capacity = floor(Base Capacity × 1.50)
```

At level 1 with the ordinary base Spell Capacity of `20`, Talent Capacity is `30`. Catalyst, Grimoire, equipment, and temporary capacity bonuses do not increase Talent Capacity unless a rule explicitly targets racial talents.

Using the completed talent costs one Standard Action but has no Mana, Stamina, HP, ammunition, item, cooldown, or additional resource cost. Cost and Sacrifice Nodes that would add one of those costs are invalid in this racial graph. All other targeting and legality rules represented by its chosen Nodes still apply.

The talent grows without changing its immutable graph:

```text
Growth Steps = floor(Character Level / 10)
Talent Output Multiplier = 1 + Growth Steps × 1%
```

The multiplier applies once to positive damage, healing, Shield, resource restoration, Status Potency, and Status Counter produced by the talent. Each output is floored after multiplication. It does not increase target count, action count, trigger count, or create additional Nodes. There is no racial growth cap beyond the level cap.

An invalid or over-capacity graph is rejected before confirmation with the offending component and required/available capacity. After confirmation, a missing mod-owned component produces a contextual load error; it does not silently replace the component or permit rebuilding the talent.

## Demon

### Passive — Pavor da Sociedade

An enemy qualifies while it has at least one active effect in the `Debuff`, `Crowd Control`, or `DoT` category. An enemy with several qualifying effects is counted once. Defeated, removed, allied, and neutral entities are not counted.

At the start of each Demon turn, let `N` be the number of qualifying enemies currently on the field. The Demon recovers:

```text
HP Restored = floor(Max HP × N / 100)
Mana Restored = floor(Max Mana × N / 100)
```

This is `1%` of each maximum resource per qualifying enemy. Recovery cannot exceed the normal resource maximum. There is no racial cap on `N`; every qualifying enemy on the field contributes.

While `N > 0`, the Demon also has source-bound Haste and Rage with:

```text
Haste Potency = 2 × N
Rage Potency = 2 × N
```

This means `+2%` Evasion, `+2%` Priority, and `+2%` Physical Damage per qualifying enemy under the normal Haste and Rage definitions. These racial instances have no Counter, are recalculated whenever a relevant effect or entity enters/leaves the field, and disappear at `N = 0`. They have no racial Potency cap, although the global rule preventing ordinary Evasion from reaching `100%` still applies.

### Passive — Coração Partido

A Demon begins each combat with `7` intact Hearts. Hearts are combat charges and return to `7` at the beginning of the next combat.

One Heart explodes when an HP change crosses from at least `50%` Max HP to below `50%` Max HP. Use integer comparison rather than a rounded threshold:

```text
Before: 2 × HP >= Max HP
After:  2 × HP < Max HP
```

Only one Heart can explode for a single HP-change event. Remaining below the threshold does not consume more Hearts; the Demon must first return to at least `50%` and cross below it again. With no intact Heart, the trigger does nothing.

The explosion consumes one Heart, grants Shield equal to `20%` of the Demon's Max HP, and applies `Coração Partido` as a self-Debuff:

```text
Shield HP = floor(Max HP × 0.20)
Potency = 15
Outgoing Damage = -(Potency × 1%)
Counter = 3
```

The penalty affects all damage sourced by the Demon, including Physical, Magical, True, and damage from its existing DoTs. Reapplication follows normal Potency/Counter competition and may refresh the stronger remaining instance; the damage penalties do not stack. The Shield is created during `OnHPBelow`, after the triggering HP loss and before Death Check. It does not restore HP, so a Demon at `0 HP` still requires another valid trigger to avoid death.

### Active — Fogo do Purgatório

Demons unlock the racial magic type `Dark Flame`, its Nodes, and two initial spell templates:

- **Dark Flame Armament:** imbues unarmed attacks or one equipped weapon with Dark Flame through its granted Node graph.
- **Black Flame Orb:** launches a projectile whose damage and applied DoT use Dark Flame.

The spell templates use the normal action, Weight, and resource rules encoded by their Nodes; receiving them does not make all Demon magic free.

Dark Flame has the `Fire`, `Darkness`, `Debuff`, and `DoT` tags. Its damage is True Damage. A Dark Flame DoT deals damage equal to Potency at its normal tick and loses `1 Counter` after that tick; it does not naturally lose Potency.

Dark Flame has priority over other hostile DoTs. Applying it removes active non-Dark-Flame hostile DoTs from the target before normal Dark Flame competition. While Dark Flame remains active, incoming non-Dark-Flame hostile DoTs are discarded. Dark Flame instances compete with one another using the normal Potency-then-Counter rule. Beneficial effects and non-DoT Debuffs or Crowd Control are unaffected.

Entities with the `Demon` race tag are immune to Dark Flame damage and reject its DoT application, regardless of the source. The immunity does not grant general Fire, Darkness, or True Damage immunity.

## Angel

### Passive — Santidade

Angels gain `+20%` to healing and Shield they cause and `+20%` to healing and Shield they receive. Source and recipient bonuses are additive in the shared modifier stage:

```text
Final Amount = floor(Base Amount × (1 + Source Bonus + Recipient Bonus))
```

An Angel healing a non-Angel applies `+20%`. An Angel healing themself, or one Angel healing another Angel, applies `+40%`. The modifier applies to the generated amount before HP/Shield maximum or replacement rules; it does not recover beyond the target's Max HP.

After ordinary Counter reductions, an Angel reduces the Counter of incoming hostile `Debuff`, `DoT`, and `Crowd Control` effects by `25%`:

```text
Angel Counter = max(1, floor(Adjusted Counter × 0.75))
```

This changes Counter only, not Potency. Immediate effects with no Counter are unaffected. If the effect has multiple qualifying category tags, Santidade applies only once.

### Passive — Halo

Angel affinity grants:

```text
Fire Damage: +15%
Light Damage: +15%
Body-weapon Damage: +15%
Spell Capacity: +20%
```

Fire and Light share one elemental-affinity bonus; an effect tagged with both receives `+15%`, not `+30%`. The body-weapon bonus is separate and may combine additively with the elemental bonus for `+30%`. A body weapon is a weapon explicitly tagged `body_weapon`; unarmed attacks qualify only when their action has that tag.

Halo's Spell Capacity is calculated after the normal Intelligence, Catalyst Tier, and Grimoire Rank formula:

```text
Angel Spell Capacity = floor(Normal Spell Capacity × 1.20)
```

Halo increases the Weight capacity available for Nodes; it does not reduce Node Weight or Mana cost and does not itself unlock Fire or Light ownership.

### Active — Expurgo

Expurgo is a divine single-target action that deals True Damage based on the Angel's missing HP at resolution:

```text
Missing HP = Max HP - Current HP
Base Expurgo Damage = floor(Max HP × 0.20 + Missing HP × 0.50)
```

Against a target tagged `Monster` or with race `Demon`, multiply the result by `2`. A target that is both receives the multiplier once, not twice. Expurgo follows the normal True Damage pipeline: it ignores Defense and Tenacity but still interacts with an applicable Block, Shield, and HP.

Expurgo costs one Standard Action and has no additional resource cost. It has one use per combat. The use is consumed when the action resolves successfully. If Expurgo is the damage source that eliminates its target, its use is restored immediately, allowing it to be used again later in the same combat. A miss, immunity, invalid target, or kill credited to another source does not restore the use.

## Racial Calculation Examples

```text
Human receives a 1,250 XP award:
floor(1,250 × 1.10) = 1,375 XP

Human reaches level 10:
1 normal + 1 fifth-level bonus + 2 Trabalho Duro = 4 points

Human Talent at level 250 produces a base 81 Shield:
Growth Steps = floor(250 / 10) = 25
floor(81 × 1.25) = 101 Shield
```

```text
Demon has 1,250 Max HP, 410 Max Mana, and 3 qualifying enemies:
HP = floor(1,250 × 3 / 100) = 37 restored
Mana = floor(410 × 3 / 100) = 12 restored
Haste = 6 Potency
Rage = 6 Potency

Demon with 1,000 Max HP crosses below half:
Shield = floor(1,000 × 0.20) = 200
100 outgoing damage during Coração Partido becomes 85
```

```text
Angel produces 400 healing for a non-Angel:
floor(400 × 1.20) = 480 healing

Angel heals themself for the same base amount:
floor(400 × 1.40) = 560 healing

Angel with 1,000 Max HP and 400 Current HP uses Expurgo:
Missing HP = 600
floor(1,000 × 0.20 + 600 × 0.50) = 500 True Damage
Against a Monster or Demon: 500 × 2 = 1,000 True Damage
```

## Racial Events, Saves, and Diagnostics

The racial kit uses the shared event system rather than terminal-specific rules:

| Ability | Required event or state |
| --- | --- |
| Experiência Nata | Experience award resolution |
| Trabalho Duro | Level gained |
| Talentoso | Character creation confirmation and action resolution |
| Pavor da Sociedade | Turn started; effect/entity field changes |
| Coração Partido | `OnHPBelow` during On HP Change triggers |
| Dark Flame | Status application, status tick, and damage resolution |
| Santidade | Heal/Shield generation and hostile status application |
| Halo | Offensive modifiers and Spell Capacity validation |
| Expurgo | Action resolution, `OnDamageReceived`, `OnHPDamage`, and kill attribution |

Saves must preserve the race ID and confirmed Human Talent graph. Demon Hearts and Expurgo availability are combat state, not permanent progression. Loading an unknown race or missing racial component must report its stable ID and source file rather than panic.

`explain last` must retain, when relevant: race and ability IDs, triggering event, source and target IDs, base value, qualifying-enemy count, Max/Current/Missing HP, capacity before and after the racial modifier, percentage modifiers in declaration order, pre-floor value, final value, Hearts or uses before and after, effects removed or rejected by Dark Flame priority, and the reason an activation was rejected.

Tests must cover the formulas and examples above, floor behavior, one-count-per-enemy, uncapped Demon enemy scaling, the global Evasion ceiling, exact `50%` Heart crossings, one Heart per HP event, Heart reset, Dark Flame immunity and DoT priority, Human Talent immutability and zero resource cost, Angel source/recipient stacking, Counter reduction, Halo tag combinations, Expurgo's single multiplier, use consumption, and kill restoration.

## Initial Elements

During character creation, the player may choose up to:

2 Elements

Initial elemental options:

- Fire
- Water
- Earth
- Air
- Light
- Darkness
- Poison
- Ice
- Lightning

These initial elements determine:

- which elemental Nodes the player can initially access;
- which elemental magic structures are initially available;
- which early spells the player can construct or use;
- which elemental progression paths begin unlocked.

The player's initial element selection does NOT permanently prevent access to other elements.

Additional elements can later be obtained through Elemental Trials.

## Element Ownership

Elements are persistent character progression.

A character can eventually own more elemental affinities than the two selected during character creation.

The game should distinguish between at least:

- Element unavailable
- Element unlocked
- Element awakened

Additional progression tiers may be added later.

Element state belongs to the save.

Element IDs should remain stable and moddable.
