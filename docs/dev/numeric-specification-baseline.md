# Numeric Specification Baseline

This page closes the most important non-Glyph gaps found by the numeric audit. These are canonical defaults for implementation and tests. Every value is configurable through `config.toml`, but changing it changes the ruleset version and must be recorded in `explain last` and save metadata.

## Shared combat values

All gameplay quantities are signed 64-bit integers after each named stage. Fractions are floored, never rounded conventionally. A value below zero is clamped to zero unless a rule explicitly says it is a debt. Percentage modifiers use basis points (`10_000 = 100%`). Additive modifiers are applied before multiplicative modifiers; multiplicative modifiers are multiplied in declaration order, then floored once.

Damage is composed as `Base Damage + Scaling Contributions`, producing pre-modifier damage. It then follows: Offensive Modifiers, shared Defense/Defensive Modifiers, Damage Type, Block, Shield, Tenacity, HP, On Damage triggers, On HP Change triggers, Death Check. Equipment and Attributes provide sources for Base Damage, Scaling, and modifiers; they are not artificial damage stages. Physical and Magical damage use the normal Shield/Tenacity path. True Damage bypasses Defense and Tenacity but still encounters Block only when the action is blockable, then Shield and HP. While Tenacity is positive, applicable damage is split `50% HP / 50% Tenacity`; odd points go to HP. When Tenacity is zero, all applicable damage goes to HP.

Shared Defense and penetration are now defined. Physical and Magical damage use the same Defense system; there is no separate core Magic Resistance attribute:

```text
effective_defense = max(0, Defense + conditional_defense[damage_type] - penetration[damage_type] - universal_penetration)
defense_multiplier = 10_000 / (10_000 + effective_defense)
```

The multiplier is floored after multiplication. `Defense` and penetration are integer points; the curve has no hard immunity and cannot reduce damage below `1` when positive damage reaches the Defense stage. True Damage ignores Defense and penetration but still encounters Shield and then HP.

## Attributes and resources

At level 1 every character has `100 HP`, `50 Tenacity`, and `30 Mana`. Each point grants: Vigor `+10 Max HP`, Resistance `+10 Max Tenacity`, Mana `+10 Max Mana`, and each point of Strength, Dexterity, Precision, or Intelligence produces `7 Power` for its applicable Scaling. Natural regeneration is `0` during combat; outside combat HP, Tenacity, and Mana recover `5%` of maximum per exploration minute, floored to `1` when non-full.

Resistance reduces debuff Counter with the approved curve:

```text
final_counter = max(1, floor(base_counter × 1000 / (1000 + Resistance × 10)))
```

This is applied once on application, before competition. Spirit is not a separate attribute; all magical offensive and spell-capacity behavior uses Intelligence. This resolves the former Spirit/Intelligence ambiguity without changing the seven documented core attributes.

## AP, Cast, Priority, and action order

```text
AP maximum per actor turn = 100
Standard action = 100 AP
Quick action = 50 AP
Free action = 0 AP
Bonus Action = 50 AP unless explicitly overridden

actor_priority = 100 + 2 × Dexterity
final_priority = actor_priority + action_priority + equipment_priority + active_effect_priority
```

AP does not accumulate and Priority is not AP. At combat start, the player's controlled group receives the first action opportunity; the player chooses which eligible controlled party member performs it. Thereafter, eligible actions resolve by highest Final Priority; ties resolve by player party, then stable entity ID. Cast is a round eligibility delay: `eligible_round = declared_round + Cast`. Cast actions cannot resolve before their eligible round. Maximum Bonus Action depth is `2`, maximum Bonus Actions per actor per round is `3`, and an actor cannot activate the same Bonus Action source twice in one parent action. Accuracy, Evasion, Parry, Block, Shield, and the shared Defense pipeline are defined in [Combat](../systems/combat.md#canonical-quantified-combat-contract).

## Status lifecycle

Counter decays at the end of the owning actor's turn unless the status card specifies another event. Potency decay is effect-specific; a status with no declared Potency decay retains Potency until Counter expiry. Stun, Binding, Silent, Fear, Taunt, and Charm have Potency `1` and Counter `1..5`. Bleed deals Physical damage; Burn, Poison, Electrified, and Rupture deal Magical damage; Tremor deals Physical Tenacity damage. Poison replacement keeps `50%` of old Counter only when the incoming Potency is strictly higher, capped at `20`; otherwise the incoming instance loses by normal Potency/Counter competition. Curse applies `-20%` to the affected debuff's Potency and `-20%` to its Counter, once, and cannot modify another Curse.

## Progression

The required XP to advance from level `L` is `1000 × L + 100`. Level 1 is the starting level. Each level awards one attribute point; every 5th level awards one additional point; every 10th level awards one Node/Glyph slot point. Maximum level is `7000`. Respec costs `100 × current level` Gold, has no cooldown, and cannot lower a level or remove a learned stable ID.

## Alchemy

Each ingredient contributes four integer channels: Potency, Counter, Mana, and Stability. The generated potion sums channels, then clamps each to `1..100`. Familiarity is `0..100` per exact ingredient multiset. Success chance is:

```text
chance = clamp(40 + 2 × Alchemy + floor(Familiarity / 2) + 5 × Stability - 10 × IngredientCount, 5, 95)
```

On failure all ingredients are lost. On success, potion Mana Cost is `5 + floor((Potency + Counter) / 10)`, value is `10 × (Potency + Counter) + 50 × ModifierCount`, and Duration is represented as Counter; no independent runtime Duration exists. Familiarity increases by `5` on success and `2` on failure, capped at `100`.

## Equipment Sigils

Every Sigil has Weight `3` unless its card states `5` for propagation or execution. A source may activate once per event; reserve stores cap at `3` activations; delayed releases occur at the next matching event and expire after `3 rounds`. The exclusive weapon and armor Sigils in [Sigils](../systems/sigils.md) use these concrete defaults: Opportunist `+1 immediate trigger`, Twin Echo second activation `-25%`, Finesse converts `20%` secondary Scaling, Momentum gains `+5 Poise per consecutive hit` capped at `25`, Cleave transfers `20%` overkill capped at `30`, Sundering stores `10%` Tenacity damage capped at `50`, Reach requires distance `2..4` and grants `+15%`, Draw loses `20 Priority` for `+25%` next-shot damage, Loaded stores one shot, Reversal stores `20%` blocked damage capped at `30`, Bastion gains `5 Guard per non-offensive action` capped at `20`, Reaping stores `10%` lost HP capped at `50`, and Aftershock repeats `30%` of the first impact after `1 action`.

Armor exclusive defaults are: Untouched grants `+10%` next effect after one undamaged turn, Flowstep grants `+15 Priority` after Dodge, Adaptation grants `+5%` resistance per repeated type capped at `20%`, Retention stores `25%` lost Tenacity capped at `30`, Immovable converts prevented movement at `5 Tenacity per tile` capped at `20`, and Resonance grants the first Glyph of each spell `-1 Weight` (minimum 1).

## Crafting, economy, and gathering

Crafting is a direct recipe operation. A recipe declares stable-ID inputs and stable-ID outputs; valid inputs produce the declared output. There is no generic crafting quality, output budget, failure, waste, or durability rule. A recipe may define a special exception explicitly, but the engine must not invent one.

The economy is a simple game currency. Items and services use declared prices, and activities may award declared currency amounts. There is no generic inflation simulation, regional price model, or merchant-economy subsystem unless a content definition explicitly adds one.

Mining nodes yield the amount declared by their node definition, and tool requirements are declared by the resource definition. No durability rule is implied by gathering.

## Completion rule

Any future system page that still contains only a conceptual paragraph for a rule with gameplay impact is incomplete. It must add: units, deterministic formula, bounds, invalid-input behavior, rounding point, three worked examples, event mapping, and `explain last` fields. The remaining lower-priority pages (world flavor, NPC rosters, and endgame encounter content) are content-authoring tasks, not unresolved engine contracts.
