# Planning Inventory

This inventory lists the preserved design material. Unless a page explicitly says otherwise, entries are specifications rather than implementation claims. The implemented combat subset is tracked separately in [Combat Vertical Slice Status](vertical-slice-status.md).

## Gameplay Systems

| System | Preserved scope | Document |
| --- | --- | --- |
| Character creation | Races, initial elements, and persistent element ownership | [Character Creation](../systems/character-creation.md) |
| Attributes and resources | Attributes, Power, HP, Tenacity, Mana, and Intelligence | [Attributes](../systems/attributes.md) |
| Scaling | Grades, Attribute Power, single/multiple Scaling, and Art of War Scaling | [Scaling](../systems/scaling.md) |
| Combat | Entry, rounds, actions, Priority, critical damage, equipment actions, Bonus Actions, fleeing, events, and explanations | [Combat](../systems/combat.md) |
| Status effects | Potency, Counter, competition, processing, 25 base effects, data, Lua, and diagnostics | [Status Effects](../systems/effects.md) |
| Magic Nodes and Glyphs | Spell grammar, general and elemental Glyphs, status editing, and catalysts | [Magic Nodes and Glyphs](../systems/glyphs.md) |
| Equipment Sigils | Equipment triggers plus general, weapon, and armor Sigils | [Equipment Sigils](../systems/sigils.md) |
| Equipment | Armor slots, trinkets, combat-ready equipment, and legendary weapons | [Equipment](../systems/equipment.md) |
| Inventory | Unlimited item storage; chests are for organization | [Inventory](../systems/inventory.md) |
| Crafting | Forging, learned recipes, materials, and Blacksmith Templates | [Crafting](../systems/crafting.md) |
| Economy | Currency sources, purchases, rewards, prices, and merchant behavior | [Economy](../systems/economy.md) |
| Mining | Pickaxes, deposits, resource tiers, and the no-durability rule | [Mining](../systems/mining.md) |
| Alchemy | Ingredients, success rules, generated potions, and Counter-based duration | [Alchemy](../systems/alchemy.md) |
| Fishing | Equipment, activities, locations, and rewards | [Fishing](../systems/fishing.md) |
| Companions and mounts | Independent actors, progression, and travel roles | [Companions](../systems/companions.md) |
| Fast travel | Churches, Goddess statues, and availability | [Fast Travel](../systems/fast-travel.md) |
| Character progression | Experience, levels, build sources, and natural restrictions | [Progression](../systems/progression.md) |
| Fighting Styles | Martial techniques, Style Nodes, mastery, Scaling, and cross-influence | [Fighting Styles](../systems/fighting-styles.md) |
| Elemental Trials | Element unlocking, Trial access, rewards, and awakening | [Elemental Trials](../systems/elemental-trials.md) |
| Trial of Cataclysm | Trial structure and persistent resources | [Cataclysm](../systems/cataclysm.md) |
| Arena | Participation, rewards, and magic restrictions | [Arena](../systems/arena.md) |
| Knight Shrines and The Kings | Challenges, rewards, and King encounters | [Shrines](../systems/shrines.md) |
| The Abyss | Procedural enemies, generated builds, rewards, and progression | [The Abyss](../systems/abyss.md) |
| Honor of the King | Activity rules and legendary reward relationship | [Honor of the King](../systems/honor-of-the-king.md) |
| Endgame design | Separation and continued relevance of long-term activities | [Endgame](../systems/endgame.md) |

## World Design

| Area | Preserved scope | Document |
| --- | --- | --- |
| World structure | Region principles, topology, and known regions | [Overview](../world/overview.md) |
| World data and modding | Stable IDs, definitions, registries, validation, and Lua extension points | [Data and Modding](../world/data-and-modding.md) |
| World progression | Nonlinear access, difficulty, and natural restrictions | [Progression](../world/progression.md) |
| Exploration | Open-world traversal, connections, routes, and transitions | [Travel](../world/travel.md) |
| Regions | Eight named regions and their build identities | [Regions](../world/regions.md) |
| World state | Campaign changes, faction/region state, serialization, and save compatibility | [State and Persistence](../world/state-and-persistence.md) |
| Hidden locations | Discovery, routes, conditions, and persistent knowledge | [Hidden Routes](../world/hidden-routes.md) |
| Transportation | Ships and other travel methods | [Transportation](../world/transportation.md) |
| Environment | Climate, weather, terrain, hazards, and regional pressure | [Environment](../world/environment.md) |
| Encounters | Regional enemy pools and encounter tables | [Encounters](../world/encounters.md) |
| Resources | Regional materials, goods, and distribution | [Resources](../world/resources.md) |
| Terminal representation | Simulation and presentation boundaries | [Terminal Representation](../world/terminal-rendering.md) |
| World diagnostics | Region loading, travel, state-change, and modding logs | [World Logging](../world/logging.md) |
| Lore | The provisional failed-summoning premise | [Basic Lore](../world/lore.md) |

The [world index](../world/README.md) is the shorter navigation page. The [NPC area](../npc/README.md) is reserved for future NPC and faction specifications; the migrated sources did not define a complete standalone NPC system.

## Development Material

- [Numeric Design Framework](numeric-design.md): deterministic units, rounding, modifier operations, status contracts, and approval states.
- [Technical Contracts](technical-contracts.md): authority, stable IDs, content validation, mods, saves, Lua, events, and `explain last`.
- [Numeric Coverage Audit](numeric-audit.md): numeric completeness, missing formulas, and planning priority for every system family.
- [System Design Principles](system-design.md): configuration, modding, persistence, diagnostics, separation, testing, and priorities.
- [Repository Workflow](repository-workflow.md): repository roles, documentation checks, and future Windows/Linux/musl builds.
- [AGENTS.md](../../AGENTS.md): authoritative architectural requirements.

The root README, contribution guide, MIT license, text conventions, documentation checker, and GitHub Actions workflow are also present.

## Other Repositories

The separate `ams` public/distribution repository contains its README, contribution guide, MIT license, distribution requirements, documentation workflow, checker, bug/feature/documentation issue forms, and pull request template.

The separate `wiki-ams` repository contains its README, contribution guide, MIT license, wiki index, documentation workflow, and checker.

All three directories are independent Git repositories on `main`. `dev-ams` has an initial commit and tracks `git@github.com:Axiom-1337-ts/dev-ams.git`; remote addresses and initial commits for the public and wiki repositories remain separate work.

## Closed by the combat vertical slice

- The executable now has typed action requirements/rejections for the systems present
  in the slice. Future equipment/world requirements extend that vocabulary when their
  owning systems become executable.
- Save format v2 fixes the initial ZIP layout and has v1 migration plus active-combat
  round-trip tests.
- Concrete Rust interfaces now separate terminal, app orchestration, combat systems,
  data registries, persistence, mod discovery, and the Lua command boundary.

## Deferred Specifications

The following are intentionally deferred content specifications, not current architectural gaps: Fishing timing and tables; Mining yields and resource respawn; Trial, Arena, Shrine, Cataclysm, Abyss, and Honor of the King tuning; regional encounter weights; environmental hazard schedules; resource distribution; and enemy/content progression. They become implementation requirements when their respective content is being built, unless another active system depends on them earlier.

The former Spirit/Intelligence, Curse, potion Duration/Counter, combat opening,
Bonus Action, status lifecycle, and Glyph budget decisions are canonicalized in the
relevant system documents and [Numeric Specification Baseline](numeric-specification-baseline.md).
Combat opening, Bonus Action limits, and the vertical-slice status lifecycle now have
executable regression coverage. Broader content integration and balance remain
versioned future work rather than open engine decisions.

The save ownership philosophy, opaque preservation of unknown mod-owned data, simple
optional-Lua mod structure, minimal Lua API v1, and deterministic regression-testing
requirements are canonicalized in [Technical Contracts](technical-contracts.md).
