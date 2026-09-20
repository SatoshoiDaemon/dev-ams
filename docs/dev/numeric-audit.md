# Numeric Coverage Audit

This audit distinguishes current implementation blockers from intentionally deferred content specifications. A value is a current gap only when an active system being implemented depends on it. Undecided content balance is not an architectural gap. The shared arithmetic and configuration rules are defined in [Numeric Design](numeric-design.md).

This is a status document, not an authority over system pages. The latest canonical numeric contract is [Numeric Specification Baseline](numeric-specification-baseline.md). Rows below marked as closed must not be treated as open design questions.

## Priority Levels

- **P0 — engine contract:** blocks safe implementation of several systems.
- **P1 — active implementation contract:** required by an active engine/system implementation.
- **P2 — integration contract:** required when connecting an already-defined system to another active system.
- **P3 — deferred content specification:** can be defined when that content is implemented.

## Core Systems

| System | Current numeric coverage | Main gaps | Priority |
| --- | --- | --- | --- |
| Attributes and resources | Base conversions, Intelligence identity, resource bases, regeneration, and debuff reduction are defined. | Implementation tests; secondary behavior can remain content/balance work | P1 |
| Scaling | Grade multipliers, single/multiple contributions, floor points, and pre-mitigation placement are defined. | Content-specific combinations and exceptional-grade availability are deferred until content needs them | P2 |
| Combat damage | Pipeline, floor rounding, shared Defense, typed penetration, Shield order, Tenacity split, AP, Cast, Priority, Accuracy, Evasion, Parry, and Block are defined. | Fleeing rules and implementation tests | P1 |
| Status effects | Potency/Counter model, default caps, competition, decay, damage types, Curse, Poison replacement, and resistance curve are defined. | Implementation tests and balance tuning | P1 |
| Glyphs | Canonical quantified specification now exists for all catalogued Glyphs, including Weight, status budget, limits, timing, cooldowns, validation, and test vectors. | Balance tuning may change versioned defaults; no implementation-blocking numeric gap remains. | Completed/P0 |
| Sigils | Canonical triggers, magnitudes, reserves, marks, switching, cooldowns, ownership, and event mapping are defined. | Implementation tests and balance tuning | P1 |
| Priority | Base formula, contribution coefficients, tie-breaker, Cast separation, and AP separation are defined; Priority has no balance cap. | Implementation tests | P1 |
| Bonus Actions | AP cost, insertion limits, trigger depth, per-round limits, and source repetition rules are defined. | Implementation tests and content-specific triggers | P1 |
| Character progression | XP curve, level rewards, maximum level, and respec costs are defined. | Implementation tests and content progression | P1 |
| Fighting Styles | Mastery curve, Node costs, cross-influence coefficients, and unlock thresholds are defined. | Implementation tests and content authoring | P1 |
| Equipment | Slots, item budgets, level growth, material coefficients, weight, requirements, armor, and penetration are defined. | Implementation tests and content authoring | P1 |
| Crafting | Crafting is a direct recipe operation: inputs produce the declared output; templates are learned stable IDs. | Recipe data schema and validation only; no generic quality, budget, or failure system | P2 |
| Inventory | Inventory is unlimited and has no categories, capacity, weight, or maximum stack size. Chests organize items. | Persistence/location schema only | P2 |
| Economy | Economy is a simple game currency with sources, purchases, and declared prices. | Currency/item price data and validation only; no inflation simulation | P2 |
| Mining | Tool gating and no durability are defined. | Yield, node availability, rarity, respawn, and tool tiers are deferred content specification | P3 |
| Alchemy | Ingredient channels, success formula, generated potion values, and Counter-based duration are defined. | Implementation tests and balance tuning | P1 |
| Fishing | Loop, tools, locations, and rewards exist. | Catch timing, rarity, success, bait, regional tables, and reward value are deferred content specification | P3 |
| Companions and mounts | The player controls companion actions; companions receive `60%` of defeated-monster XP; dead companions return after fast travel, rest, or player death; mounts use land, water, and flight traversal rules. | Persistence and combat/exploration integration tests | P2 |
| Fast travel | A point becomes `unlocked` when reached; unlocked points can be selected. Cost, cooldown, time, risk, and generic extra requirements are all `0`/absent. | None for the current contract | Completed/P1 |

## Trials and Endgame

| System | Current numeric coverage | Main gaps | Priority |
| --- | --- | --- | --- |
| Elemental Trials | Unlock and awakening structure exists. | Entry thresholds, encounter scaling, reward magnitude, and repeat rules are deferred content specification | P3 |
| Trial of Cataclysm | Activity and persistent resource concept exist. | Run scaling, resource gain/spend rates, loss/retention, and reward curve are deferred content specification | P3 |
| Arena | Activity, rewards, magic restriction, and 100-opponent structure exist. | Progression, enemy scaling, and rewards are deferred content specification | P3 |
| Knight Shrines and The Kings | Challenge and reward structure exists. | Scaling, unlocks, reward budgets, and encounter progression are deferred content specification | P3 |
| The Abyss | Procedural build structure and infinite progression intent exist. | Floor scaling, generation budgets, rewards, and failure/retention are deferred content specification | P3 |
| Honor of the King | Five-opponent activity and Excalibur reward relationship exist. | Entry, scaling, completion, and repeat rules are deferred content specification | P3 |
| Endgame | Activities are intended to remain distinct and relevant. There is no global vertical or horizontal progression cap. | Shared reward baseline and catch-up behavior are deferred content specification; individual mechanics may define natural limits | P3 |

## World Systems

| Area | Current numeric coverage | Main gaps | Priority |
| --- | --- | --- | --- |
| Regional difficulty | Nonlinear philosophy and a level example exist. | Threat budgets, enemy bands, scaling boundaries, and warning rules are deferred content specification | P3 |
| Encounters | Regional tables are structurally defined. | Spawn weights, group budgets, cooldowns, rare encounters, and state modifiers are deferred content specification | P3 |
| Environment | A configurable intensity example exists. | Hazard intervals, buildup, resistance interaction, and weather transition rates are deferred content specification | P3 |
| Resources | Regional identity is defined. | Distribution weights, depletion/respawn, rarity, and economy value are deferred content specification | P3 |
| Travel and transportation | Region-specific Fast Travel connections are defined. Generic cost, time, risk, and capacity do not exist. | Additional routes/methods are deferred content specification | P3 |
| World state | Persistent states and stable IDs are defined. | Thresholds, transition triggers, and reset/reversal rules are deferred until a campaign state requires them | P3 |

## Recommended Planning Order

The original audit below is preserved as historical sequencing. The current implementation baseline is [Numeric Specification Baseline](numeric-specification-baseline.md), and the current Glyph contract is [Canonical Quantified Specification](../systems/glyphs.md#canonical-quantified-specification).

1. Implement and test the canonical combat contract, including AttackContext, DamageContext, AP, Cast, Priority, Accuracy, Evasion, Parry, Block, shared Defense, Shield, and event ordering.
2. Connect equipment, spell, catalyst, Glyph, and Fighting Style Scaling through declared data sources.
3. Define and test fleeing rules plus remaining action-target validation rules.
4. Implement and test progression, direct recipes, alchemy, simple currency, gathering, and companion rules from their canonical pages.
5. Implement deferred content specifications only when the corresponding content enters development.

The earlier eight-step list is preserved by the canonical system documents and is no longer an unresolved planning sequence.

## Definition of Numerically Planned

A system is numerically planned when it has:

- named inputs and units;
- a deterministic formula or lookup rule;
- mechanic-required limits and invalid-input behavior;
- explicit rounding points;
- configuration keys and versioning expectations;
- interactions with statuses, events, and the damage pipeline where relevant;
- at least three worked examples, including an edge case;
- required regression-test vectors;
- combat/debug log fields that explain the result.
