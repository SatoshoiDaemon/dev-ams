# Status Effects

This is the complete initial Status Effect specification, including the shared status material formerly repeated in the Glyph notes. [Glyphs](glyphs.md) and other systems consume this model. Individual effect definitions, examples, the canonical table, and the summary are retained together for reference. Shared numeric representation and validation requirements are planned in [Numeric Design](../dev/numeric-design.md), with the remaining per-effect work tracked in [Status Numeric Planning](../dev/status-balance.md).

## Status Effect System

Status Effects are a fully defined core combat system.

Status Effects use two primary numerical properties:

```text
Counter
Potency
```

There is NO separate `Duration` property.

The terminology is:

```text
Counter
→ Duration / remaining lifetime of the effect.

Potency
→ Intensity / strength of the effect.
```

Therefore, when documentation, code, data schemas, Lua APIs, logs, or mods refer to the duration of a Status Effect, the canonical mechanical property is:

```text
Counter
```

Do NOT implement:

```text
Potency
Counter
Duration
```

as three independent generic Status Effect properties.

The correct generic model is:

```text
Potency
Counter
```

Some effects may not use one of these values, may have a fixed value, or may define special limits.

---

## Default Status Limits

Unless an individual Status Effect explicitly defines otherwise, the normal limits are:

```text
Potency: 0–99
Counter: 0–99
```

Conceptually:

```text
Burn
Potency: 40
Counter: 12
```

means Burn currently has:

```text
40 intensity
12 remaining duration
```

Individual Status Effects may override these limits.

Examples explicitly defined below include:

```text
Stun
Binding
Shield
Heal
```

The Status Effect engine MUST support per-effect limits rather than assuming every effect is always capped at `99 / 99`.

These limits should be exposed through the data/modding system.

---

## Status Categories

Core Status Effects are divided into four categories:

```text
DoT
Crowd Control
Buff
Debuff
```

These categories should exist as explicit tags/categories in the Status Effect system.

They are mechanically useful for:

```text
Nodes
Glyphs
equipment
resistances
Tenacity
Curse
cleansing
modding
targeting
conditional effects
```

Mods must be able to inspect these categories.

The engine should not determine categories by checking individual effect IDs.

Prefer:

```text
effect.has_category("debuff")
```

over logic equivalent to:

```text
if effect == Poison ||
   effect == Tremor ||
   effect == Fragile ...
```

---

## Status Effect Data Model

Status Effects should be data-driven.

A Status Effect definition should be capable of defining properties such as:

```text
ID
Name
Category
Potency Limit
Counter Limit
Fixed Potency
Uses Potency
Uses Counter
Tick Behavior
Trigger Behavior
Modifiers
Tags
```

Simple Status Effects should be expressible without requiring dedicated Rust code.

More complex behavior may use the embedded Lua API.

The base-game Status Effects defined below are canonical defaults, but their numerical values and behavior should remain exposed to the modding/configuration architecture where practical.

The goal is:

> Defined by default does not mean hardcoded forever.

---

## Status Application Competition

When competing instances of the same Status Effect are applied to the same target, they do NOT automatically stack as independent copies.

Compare:

```text
1. Potency
2. Counter
```

Potency has priority.

Counter is the tie-breaker.

The weaker instance is discarded.

Example:

```text
Existing Burn:
Potency 20
Counter 30

Incoming Burn:
Potency 25
Counter 5
```

The incoming Burn wins because:

```text
25 Potency > 20 Potency
```

even though its Counter is lower.

Another example:

```text
Existing Burn:
Potency 25
Counter 10

Incoming Burn:
Potency 25
Counter 20
```

Potency is tied, therefore Counter decides.

The incoming effect wins.

Do not combine these into:

```text
Potency 50
Counter 30
```

unless a specific mechanic explicitly changes the normal Status Effect application rule.

---

## Status Processing

Status Effects may respond to different events.

Relevant generic events include:

```text
Round Start
Action Declaration
Before Action
Attack
Evade
Damage
HP Change
End of Turn
End of Round
Status Applied
Status Removed
```

Each Status Effect defines which events matter to it.

Do not assume every Status Effect simply performs:

```text
Counter -= 1
```

at the same universal stage.

Several effects have explicitly different decay rules.

The Status Effect system therefore requires effect-specific tick behavior while still using a generic event/status architecture.

---

## DoT — Burn

Category:

```text
DoT
```

Burn deals Magical Damage.

For every:

```text
1 Potency
```

Burn deals:

```text
1 Magical Damage
```

Therefore:

```text
Burn Damage = Potency
```

Example:

```text
Burn:
Potency 30
Counter 12

Damage this turn:
30 Magical Damage
```

At every turn, Burn loses:

```text
1 Potency
1 Counter
```

Therefore:

```text
Before:
30 Potency / 12 Counter

After Tick:
29 Potency / 11 Counter
```

Burn naturally becomes weaker while simultaneously approaching expiration.

---

## DoT — Poison

Category:

```text
DoT
```

Poison deals Magical Damage.

Its damage increases according to how long the target has continuously remained Poisoned.

Formula:

```text
Poison Damage =
Potency × Turns Under Poison
```

Example:

```text
Potency = 10

First Poison Turn:
10 × 1 = 10 Magical Damage

Second Poison Turn:
10 × 2 = 20 Magical Damage

Third Poison Turn:
10 × 3 = 30 Magical Damage
```

The system must therefore track:

```text
Turns Under Poison
```

for the active Poison instance.

Every turn, Poison loses:

```text
half of its Counter
```

Fractional results follow the global rule:

```text
round down
```

Example:

```text
Counter 20
→ 10
→ 5
→ 2
→ 1
→ 0
```

Do not replace Poison's decay with ordinary `Counter - 1`.

---

## DoT — Sinking

Category:

```text
DoT
```

Sinking attacks the target's energy resource rather than directly damaging HP.

Every turn:

```text
Resource Loss = Potency
```

The affected resource is:

```text
Energy / Mana
```

according to the target's applicable resource system.

Example:

```text
Sinking:
Potency 15
Counter 8

Target loses:
15 Energy/Mana
```

At the end of the turn:

```text
Counter -= 1
```

Sinking does not normally lose Potency through its own decay rule.

---

## DoT — Electrified

Category:

```text
DoT
```

Electrified deals Magical Damage.

Its damage depends on the number of currently Electrified targets on the battlefield.

For each Potency point, damage is multiplied by the number of targets currently affected by Electrified.

Formula:

```text
Electrified Damage =
Potency × Electrified Targets On Field
```

Example:

```text
Target Potency:
20

Electrified targets currently on field:
4

Damage:
20 × 4
= 80 Magical Damage
```

At the end of the turn:

```text
Counter -= 1
```

This makes Electrified inherently stronger in encounters where it can be spread across multiple entities.

The target count must be evaluated from the actual current combat state.

---

## DoT — Bleed

Category:

```text
DoT
```

Bleed is event-driven rather than a normal automatic damage tick.

Whenever the affected target:

```text
attacks
OR
successfully evades
```

the target receives damage based on Bleed Potency.

Formula:

```text
Bleed Damage = Potency
```

Example:

```text
Bleed Potency = 24

Target attacks:
→ receives 24 Bleed Damage

Target later evades:
→ receives another 24 Bleed Damage
```

At the end of every turn, Bleed loses:

```text
half of Potency
half of Counter
```

Fractional results round down.

Example:

```text
Before:
31 Potency
17 Counter

After:
15 Potency
8 Counter
```

Bleed therefore becomes rapidly weaker if the target survives long enough.

---

## Crowd Control — Stun

Category:

```text
Crowd Control
```

While Stunned, the target:

```text
cannot attack
cannot evade
```

Stun has special limits:

```text
Potency = fixed at 1
Maximum Counter = 5
```

Therefore Stun does NOT use the normal `99 / 99` limit.

Its effective representation is:

```text
1 / 1
1 / 2
1 / 3
1 / 4
1 / 5
```

depending on remaining Counter.

Effects attempting to increase Stun Potency should have no normal effect unless they explicitly override Stun's fixed-Potency rule.

---

## Crowd Control — Binding

Category:

```text
Crowd Control
```

While Bound, the target cannot execute:

```text
movement
physical actions
```

The target may still use:

```text
long-range actions
magic
```

Binding uses:

```text
Potency = fixed at 1
Maximum Counter = 5
```

Canonical maximum:

```text
1 / 5
```

Binding is therefore restrictive without being equivalent to Stun.

---

## Crowd Control — Silent

Category:

```text
Crowd Control
```

While Silenced, the target:

```text
cannot cast magic
```

The target may still:

```text
move
perform physical actions
use applicable non-magical actions
```

Silent specifically disables spellcasting.

Do not treat Silent as Stun.

---

## Crowd Control — Fear

Category:

```text
Crowd Control
```

Fear records the entity responsible for applying it.

While affected, the target:

```text
cannot attack the entity that applied Fear
```

Fear also reduces Evasion.

For every:

```text
1 Potency
```

the target receives:

```text
-1% Evasion
```

Formula:

```text
Evasion Penalty =
Potency × 1%
```

At the end of the turn:

```text
Counter -= 1
```

Because Fear depends on its source, the active effect must preserve its source entity ID.

---

## Crowd Control — Taunt

Category:

```text
Crowd Control
```

Taunt records the entity responsible for applying it.

While Taunted, the affected target:

```text
may attack only the entity that applied Taunt
```

Additionally, the affected target deals less damage.

For every:

```text
1 Potency
```

the affected target deals:

```text
1% less damage
```

Formula:

```text
Damage Penalty =
Potency × 1%
```

At the end of each turn:

```text
Counter -= 1
```

The effect must preserve the Taunt source.

---

## Crowd Control — Charm

Category:

```text
Crowd Control
```

Charm records its source.

While Charmed, the affected target:

```text
cannot attack the entity that applied Charm
```

Additionally, the entity that applied Charm deals increased damage against the Charmed target.

For every:

```text
1 Potency
```

the Charm source deals:

```text
+1% damage
```

against that target.

Formula:

```text
Source Damage Bonus Against Charmed Target =
Potency × 1%
```

This bonus belongs specifically to the relationship:

```text
Charm Source → Charmed Target
```

It is not a universal damage increase against every enemy.

---

## Buff — Protect

Category:

```text
Buff
```

Protect grants Damage Reduction.

For every:

```text
1 Potency
```

the target receives:

```text
1% Damage Reduction
```

Formula:

```text
Damage Reduction =
Potency × 1%
```

At the end of each turn:

```text
Counter -= 1
```

Protect remains subject to the global rule that generic Damage Reduction cannot reach 100%.

---

## Buff — Haste

Category:

```text
Buff
```

Haste improves:

```text
Evasion
Priority
```

For every:

```text
1 Potency
```

the target receives:

```text
+1% Evasion
```

and increases Priority in the same proportional amount.

Conceptually:

```text
Evasion Bonus = Potency × 1%
Priority Increase = Potency × 1%
```

At the end of each turn:

```text
Counter -= 1
```

Evasion remains subject to its global maximum and can never reach 100% through ordinary scaling.

---

## Buff — Rage

Category:

```text
Buff
```

Rage increases Physical Damage.

For every:

```text
1 Potency
```

the target deals:

```text
+1% Physical Damage
```

Formula:

```text
Physical Damage Bonus =
Potency × 1%
```

At the end of each turn:

```text
Counter -= 1
```

---

## Buff — Overdrive

Category:

```text
Buff
```

Overdrive increases Mana regeneration.

For every:

```text
1 Potency
```

Mana regeneration increases by:

```text
1%
```

Formula:

```text
Mana Regeneration Bonus =
Potency × 1%
```

At the end of each turn:

```text
Counter -= 1
```

---

## Buff — Sage

Category:

```text
Buff
```

Sage increases Magical Damage.

For every:

```text
1 Potency
```

the target deals:

```text
+1% Magical Damage
```

Formula:

```text
Magical Damage Bonus =
Potency × 1%
```

At the end of each turn:

```text
Counter -= 1
```

---

## Buff — Heal

Category:

```text
Buff
```

Heal restores HP immediately.

For every:

```text
1 Potency
```

Heal restores:

```text
1% of Maximum HP
```

Formula:

```text
Healing =
Maximum HP × (Potency / 100)
```

Fractional results round down.

Heal has:

```text
NO Counter
```

It is an immediate effect rather than a persistent timed Status Effect.

Example:

```text
Maximum HP = 2000
Heal Potency = 20

Healing:
2000 × 0.20
= 400 HP
```

After resolving, Heal does not remain as an active timed status.

---

## Buff — Shield

Category:

```text
Buff
```

Shield creates a separate defensive HP pool.

Damage absorbed by Shield is taken before:

```text
Tenacity
HP
```

For every:

```text
1 Potency
```

Shield receives:

```text
10 Shield HP
```

Formula:

```text
Shield HP =
Potency × 10
```

Shield has:

```text
NO Potency limit
```

Unlike ordinary Status Effects, Shield Potency is not capped at 99.

The Shield's generated HP is a distinct resource and should be tracked explicitly.

Example:

```text
Shield Potency = 250

Shield HP:
250 × 10
= 2500
```

Shield absorbs damage before normal HP/Tenacity processing according to the global damage pipeline.

---

## Buff — Blessing

Category:

```text
Buff
```

Blessing increases:

```text
Healing received/effectiveness
Shield HP generated
```

For every:

```text
1 Potency
```

these effects increase by:

```text
1%
```

Formula:

```text
Healing Modifier =
+Potency × 1%

Shield HP Modifier =
+Potency × 1%
```

At the end of each turn:

```text
Counter -= 1
```

---

## Debuff — Fragile

Category:

```text
Debuff
```

Fragile increases damage received directly by HP.

For every:

```text
1 Potency
```

the target's HP receives:

```text
+1% damage
```

Formula:

```text
HP Damage Taken Increase =
Potency × 1%
```

Fragile modifies damage received by HP specifically.

It is therefore distinct from Tremor.

Fragile loses Counter according to its Potency:

```text
Counter Loss Per Turn = Potency
```

Therefore:

```text
Counter -= Potency
```

Example:

```text
Fragile:
Potency 8
Counter 30

After one turn:
Potency 8
Counter 22
```

---

## Debuff — Tremor

Category:

```text
Debuff
```

Tremor increases general incoming damage.

For every:

```text
1 Potency
```

the target receives:

```text
+1% Damage
```

Formula:

```text
Damage Taken Increase =
Potency × 1%
```

At the end of each turn:

```text
Counter -= 1
```

Unlike Fragile, Tremor is not specifically limited to the HP portion of damage.

---

## Debuff — Rupture

Category:

```text
Debuff
```

Rupture causes incoming attacks to ignore part of the target's Tenacity.

For every:

```text
1 Potency
```

attacks ignore:

```text
1% Tenacity
```

while Rupture remains active.

Formula:

```text
Tenacity Ignore =
Potency × 1%
```

At the end of each turn:

```text
Potency -= 1
```

Rupture decays through Potency rather than Counter.

Do not automatically convert this to `Counter -= 1`.

---

## Debuff — Exhaust

Canonical name:

```text
Exhaust
```

Category:

```text
Debuff
```

Exhaust decreases:

```text
Attack Speed
Evasion
```

For every:

```text
1 Potency
```

the affected values decrease by:

```text
1%
```

Formula:

```text
Attack Speed Penalty =
Potency × 1%

Evasion Penalty =
Potency × 1%
```

At the end of each turn:

```text
Counter -= 1
```

Exhaust also participates in Priority calculations where Attack Speed/action speed is relevant.

---

## Debuff — Attack Down

Category:

```text
Debuff
```

Attack Down reduces damage dealt.

For every:

```text
1 Potency
```

the affected target deals:

```text
1% less damage
```

Formula:

```text
Damage Dealt Penalty =
Potency × 1%
```

At the end of each turn:

```text
Counter -= 1
```

---

## Debuff — Curse

Category:

```text
Debuff
```

Curse is a modifier for other Debuffs.

Its strength is based on:

```text
Counter
```

rather than Potency for this interaction.

For every:

```text
1 Counter
```

Curse modifies the efficiency of other Debuffs by:

```text
1%
```

The currently supplied design states that Curse:

```text
increases the efficiency of other Debuffs
and decreases the efficiency of other Debuffs
by 1% per Counter
```

These two operations conflict as currently written.

Do NOT invent which second target/category was intended.

Keep the behavior configurable/data-driven and mark the second modifier as requiring clarification before implementing the final base-game Curse behavior.

The confirmed decay rule is:

```text
Counter -= 1 per turn
```

---

## Canonical Status Table

The current base-game Status Effects are:

```text
DoT
├── Burn
├── Poison
├── Sinking
├── Electrified
└── Bleed

Crowd Control
├── Stun
├── Binding
├── Silent
├── Fear
├── Taunt
└── Charm

Buff
├── Protect
├── Haste
├── Rage
├── Overdrive
├── Sage
├── Heal
├── Shield
└── Blessing

Debuff
├── Fragile
├── Tremor
├── Rupture
├── Exhaust
├── Attack Down
└── Curse
```

These are canonical base-game effects.

Mods may add additional effects and may alter their behavior through supported modding/configuration mechanisms.

---

## Status Summary

Canonical behavior summary:

```text
Burn
→ Magical Damage = Potency
→ -1 Potency / -1 Counter per turn

Poison
→ Magical Damage = Potency × Turns Poisoned
→ Counter halved per turn

Sinking
→ Energy/Mana Loss = Potency
→ -1 Counter per turn

Electrified
→ Magical Damage = Potency × Electrified Targets
→ -1 Counter per turn

Bleed
→ Damage = Potency whenever target attacks or evades
→ Potency and Counter halved per turn

Stun
→ Cannot attack or evade
→ Potency fixed 1
→ Counter max 5

Binding
→ Cannot move/use physical actions
→ Ranged and Magic remain available
→ Potency fixed 1
→ Counter max 5

Silent
→ Cannot cast Magic

Fear
→ Cannot attack source
→ -1% Evasion per Potency
→ -1 Counter per turn

Taunt
→ Can attack only source
→ -1% Damage dealt per Potency
→ -1 Counter per turn

Charm
→ Cannot attack source
→ Source deals +1% Damage per Potency against target

Protect
→ +1% Damage Reduction per Potency
→ -1 Counter per turn

Haste
→ +1% Evasion and proportional Priority per Potency
→ -1 Counter per turn

Rage
→ +1% Physical Damage per Potency
→ -1 Counter per turn

Overdrive
→ +1% Mana Regeneration per Potency
→ -1 Counter per turn

Sage
→ +1% Magical Damage per Potency
→ -1 Counter per turn

Heal
→ Heal 1% Max HP per Potency
→ No Counter

Shield
→ 10 Shield HP per Potency
→ No Potency limit

Blessing
→ +1% Healing and Shield HP per Potency
→ -1 Counter per turn

Fragile
→ +1% Damage received by HP per Potency
→ Counter decreases by Potency per turn

Tremor
→ +1% general Damage received per Potency
→ -1 Counter per turn

Rupture
→ Ignore 1% Tenacity per Potency
→ -1 Potency per turn

Exhaust
→ -1% Attack Speed/Evasion per Potency
→ -1 Counter per turn

Attack Down
→ -1% Damage dealt per Potency
→ -1 Counter per turn

Curse
→ modifies other Debuff efficiency by Counter
→ -1 Counter per turn
→ exact second efficiency interaction requires clarification
```

---

## Status Modding Requirements

Although the base Status Effects are defined, they MUST NOT be implemented as an entirely closed hardcoded system.

Mods must be capable of:

```text
adding Status Effects
changing Potency limits
changing Counter limits
changing decay rules
changing trigger events
changing damage formulas
changing categories
changing modifiers
changing fixed values
adding custom Lua behavior
```

Whenever practical, the base effects should use the same generic infrastructure available to mods.

A mod should be capable of defining something conceptually similar to:

```text
Status:
    ID: example:frostbite
    Category: Debuff
    PotencyLimit: 99
    CounterLimit: 99

    Modifier:
        Evasion: -0.5% per Potency

    OnEndTurn:
        Counter -= 1
```

without modifying Rust.

Complex effects may register Lua callbacks.

---

## Status Lua API

The Lua API should eventually support operations conceptually equivalent to:

```text
status.register(...)
status.apply(...)
status.remove(...)
status.get(...)
status.get_potency(...)
status.get_counter(...)
status.set_potency(...)
status.set_counter(...)
status.modify_potency(...)
status.modify_counter(...)
status.has_category(...)
```

and event hooks such as:

```text
on_status_applied
on_status_removed
on_status_tick
on_attack
on_evade
on_damage
on_round_end
```

Names are illustrative.

The important requirement is that mods can reproduce mechanics comparable in complexity to:

```text
Poison
Bleed
Electrified
Charm
Rupture
Curse
```

without requiring a custom Rust patch.

---

## Status Logging

Status calculations should be visible in detailed combat logs.

Example:

```text
[COMBAT] Burn triggered on Ancient Dragon.

Potency: 24
Counter: 8

Magical Damage:
24

Decay:
Potency 24 → 23
Counter 8 → 7
```

For Poison:

```text
[COMBAT] Poison triggered on Ancient Dragon.

Potency: 15
Turns Poisoned: 4

Damage:
15 × 4 = 60 Magical Damage

Counter:
18 → 9
```

For Bleed:

```text
[COMBAT] Ancient Dragon attacked while Bleeding.

Bleed Potency: 30
Triggered Damage: 30

Cause:
Attack
```

Status effects should never be mysterious numerical changes when detailed combat logging is enabled.

---

## Status System Rule

The central Status Effect rule is:

> Counter determines how long an effect persists. Potency determines how strong it is — except where the individual effect explicitly defines a different use.

Effects are allowed to manipulate these properties differently.

Therefore:

```text
Burn
→ loses both

Poison
→ halves Counter

Bleed
→ halves both

Rupture
→ loses Potency

Heal
→ has no Counter

Shield
→ has unlimited Potency
```

The generic engine must support these differences.

Do not normalize every Status Effect into the same `Potency + Duration - 1 per turn` model.

## Unresolved Effect Details

Curse's conflicting second modifier is explicitly unresolved above. The source does not assign an explicit decay event or decay amount to every effect (including Stun, Binding, Silent, Charm, and Shield), and does not specify a damage type for Bleed. Do not infer a universal decay rule or damage type from the category or from another effect. Poison's handling of accumulated poisoned turns when a competing instance replaces it is also not specified.
