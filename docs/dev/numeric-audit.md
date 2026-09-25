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
| Attributes and resources | Base conversions, Power, HP, Tenacity, and Mana are implemented and regression-tested for the slice; regeneration and secondary behavior remain canonical contracts. | Connect future subsystem/content uses and add their tests when executable | Implemented slice/P2 |
| Scaling | Grade multipliers, multiple declared sources, floor points, and pre-mitigation placement are implemented and tested. | Connect equipment, catalyst, Glyph, and Fighting Style sources when those systems become executable | Implemented slice/P2 |
| Combat damage | Pipeline, floor rounding, shared Defense, Shield order, Tenacity split, AP, Cast, Priority, Accuracy, Evasion, Parry, Block, Fleeing, events, and delayed death confirmation are implemented and tested. | Broader content and balance vectors only | Implemented slice/P2 |
| Status effects | Potency/Counter competition, Resistance, ticks, decay, removal, immunity, and the base data catalog are implemented and tested. | Complete content-specific semantics outside the demonstration and continue balance tuning | Implemented slice/P2 |
| Glyphs | The canonical catalog, registry, validation, and spell-template references exist. | Executable semantics remain partial where Glyphs depend on editors, equipment, spatial rules, exploration, or other future systems | Partial/P2 |
| Sigils | Canonical triggers, magnitudes, reserves, marks, switching, cooldowns, ownership, and event mapping are defined. | Implementation tests and balance tuning | P1 |
| Priority | Base formula, contribution coefficients, stable tie-breakers, Cast separation, and AP separation are implemented and tested; Priority has no balance cap. | Broader content vectors only | Implemented slice/P2 |
| Bonus Actions | AP cost, queue insertion, depth, per-round limits, and source repetition rules are implemented and tested. | Content-specific trigger definitions | Implemented slice/P2 |
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

The current implementation baseline is [Numeric Specification Baseline](numeric-specification-baseline.md), and the current Glyph contract is [Canonical Quantified Specification](../systems/glyphs.md#canonical-quantified-specification).

1. Verify the portable Windows and static-musl artifacts on remote CI and preserve
   deterministic copied-directory smoke tests.
2. Extend declarative requirements and unresolved-reference handling when equipment
   and inventory become executable.
3. Connect equipment, spell, catalyst, Glyph, and Fighting Style Scaling through the
   existing declared data-source pipeline.
4. Implement the non-spatial Glyph/editor subset, then add spatial primitives only
   alongside an executable exploration/world slice.
5. Implement progression, direct recipes, alchemy, simple currency, gathering, and
   companion rules from their canonical pages when those systems enter development.
6. Define deferred balance/content values only when the corresponding content is
   actively being built.

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
