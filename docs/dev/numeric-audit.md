# Numeric Coverage Audit

This audit identifies which systems already have usable numbers and which require structural or balancing work. It does not make unresolved values canonical. The shared arithmetic and configuration rules are defined in [Numeric Design](numeric-design.md).

## Priority Levels

- **P0 — engine contract:** blocks safe implementation of several systems.
- **P1 — core balance:** required for a playable combat and progression loop.
- **P2 — content economy:** required for sustainable activities and world rewards.
- **P3 — later activity tuning:** can follow a working core loop.

## Core Systems

| System | Current numeric coverage | Main gaps | Priority |
| --- | --- | --- | --- |
| Attributes and resources | Base conversions are defined: `10` HP/Tenacity/MP and `7` Power. | Spirit/Intelligence decision, resource bases, caps, regeneration, soft caps, diminishing returns | P0 |
| Scaling | Grade multipliers D through SSS are defined. | Combination of spell, catalyst, weapon, and Art of War Scaling; exceptional-grade availability | P0 |
| Combat damage | Pipeline, floor rounding, damage types, Shield order, and Tenacity split are defined. | Defensive formulas, modifier stacking, armor/penetration, damage bounds | P0 |
| Status effects | Potency/Counter model, default caps, competition, and many effect formulas are defined. | Missing decay events, Bleed damage type, Curse conflict, Poison replacement history, resistance curve | P0 |
| Glyphs | Behaviors and a few illustrative Weight/status examples exist. | Weight equation, conversion efficiency, limits, target/repeat caps, timing, cooldowns, status mutation rules | P0 |
| Sigils | Triggers and behaviors are proposed. | Every magnitude, reserve, mark, switching, cooldown, ownership, and event-mapping rule | P1 |
| Priority | Influencing factors and example calculations exist. | Base formula, contribution coefficients, tie-breaker, bounds, cast/action/equipment Weight mapping | P1 |
| Bonus Actions | Insertion behavior is defined. | Trigger depth, per-action/per-round limits, ordering, cycle detection | P0 |
| Character progression | Long-term philosophy and sources are defined. | XP curve, level rewards, caps/soft caps, respec costs, source scaling | P1 |
| Fighting Styles | Structure and Scaling relationship exist. | Mastery curve, Node costs, cross-influence coefficients, unlock thresholds | P1 |
| Equipment | Slot counts and combat-ready limits are defined. | Base item budgets, level growth, material coefficients, durability decision, armor and penetration | P1 |
| Crafting | Inputs, templates, and outputs are defined conceptually. | Material quantities, quality ranges, recipe costs, output budget, failure/waste rules | P2 |
| Inventory | Limited-category philosophy is defined. | Category limits, Weight/capacity rules, stack sizes | P2 |
| Economy | Sources and sinks are listed. | Currency scale, price bands, merchant modifiers, reward curves, inflation controls | P2 |
| Mining | Tool gating and no durability are defined. | Yield, node availability, rarity, respawn, tool tiers | P2 |
| Alchemy | Generated properties and success concept exist. | Ingredient contribution, success formula, potion Weight/value, Duration/Counter mapping | P1 |
| Fishing | Loop, tools, locations, and rewards exist. | Catch timing, rarity, success, bait, regional tables, reward value | P2 |
| Companions and mounts | Independent actor and progression roles exist. | Active limits, growth, action economy, summon/mount coefficients | P1 |
| Fast travel | Valid destination types exist. | Unlock conditions, costs, cooldowns, restrictions | P3 |

## Trials and Endgame

| System | Current numeric coverage | Main gaps | Priority |
| --- | --- | --- | --- |
| Elemental Trials | Unlock and awakening structure exists. | Entry thresholds, encounter scaling, reward magnitude, repeat rules | P2 |
| Trial of Cataclysm | Activity and persistent resource concept exist. | Run scaling, resource gain/spend rates, loss/retention, reward curve | P3 |
| Arena | Activity, rewards, and magic restriction exist. | Matchmaking bands, rating, enemy scaling, reward schedule | P3 |
| Knight Shrines and The Kings | Challenge and reward structure exists. | Shrine scaling, unlocks, reward budgets, King encounter progression | P3 |
| The Abyss | Procedural builds and progression exist. | Floor scaling, generation budgets, reward curve, failure/retention | P3 |
| Honor of the King | Activity and reward relationship exist. | Entry condition, challenge scaling, completion/repeat rules | P3 |
| Endgame | Activities are intended to remain distinct and relevant. | Shared reward baseline, horizontal/vertical progression limits, catch-up behavior | P3 |

## World Systems

| Area | Current numeric coverage | Main gaps | Priority |
| --- | --- | --- | --- |
| Regional difficulty | Nonlinear philosophy and a level example exist. | Threat budgets, enemy bands, scaling boundaries, warning rules | P2 |
| Encounters | Regional tables are structurally defined. | Spawn weights, group budgets, cooldowns, rare encounters, state modifiers | P2 |
| Environment | A configurable intensity example exists. | Hazard intervals, buildup, resistance interaction, weather transition rates | P2 |
| Resources | Regional identity is defined. | Distribution weights, depletion/respawn, rarity, economy value | P2 |
| Travel and transportation | Connections and methods are defined. | Travel time, cost, danger, route modifiers, vehicle capacity | P3 |
| World state | Persistent states and stable IDs are defined. | Thresholds, transition triggers, reset/reversal rules | P2 |

## Recommended Planning Order

1. Approve numeric representations, rounding boundaries, modifier operations, and logging records.
2. Resolve Intelligence versus Spirit and complete the attribute/resource contract.
3. Complete the status definition table, including every tick, decay, expiry, and damage type.
4. Approve the Glyph Weight and status-budget model, then assign parameters and bounded behavior to every Glyph.
5. Define Priority, Bonus Action depth, damage modifiers, defense, armor, and penetration.
6. Connect equipment, spell, catalyst, and Fighting Style Scaling.
7. Define XP, item budgets, crafting, alchemy, economy, gathering, and companion growth.
8. Tune world and endgame systems only after the core combat model produces stable test vectors.

## Definition of Numerically Planned

A system is numerically planned when it has:

- named inputs and units;
- a deterministic formula or lookup rule;
- minimums, maximums, and invalid-input behavior;
- explicit rounding points;
- configuration keys and versioning expectations;
- interactions with statuses, events, and the damage pipeline where relevant;
- at least three worked examples, including an edge case;
- required regression-test vectors;
- combat/debug log fields that explain the result.
