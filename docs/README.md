# Initial Development Documentation

This is the initial design reference for A Magic Sovereign. Each independent system has a dedicated Markdown document; related internal mechanics stay together. All documentation is in English.

These documents describe design requirements and proposals, not an implemented game. Preserve tentative language and explicit open questions. The repository's [architectural requirements](../AGENTS.md) still apply; conflicts between initial drafts and those requirements are called out rather than silently resolved.

## Gameplay Systems

| System | Document |
| --- | --- |
| Character creation, races, and elemental ownership | [Character creation](systems/character-creation.md) |
| Attributes, Power, HP, Tenacity, and other resources | [Attributes](systems/attributes.md) |
| Attribute-to-damage scaling | [Scaling](systems/scaling.md) |
| Turn order, actions, critical damage, fleeing, and calculation logs | [Combat](systems/combat.md) |
| Status rules, Potency, Counter, competition, effects catalog, events, and Lua | [Status effects](systems/effects.md) |
| Magic grammar, elemental Glyphs, status editor, and catalyst specializations | [Magic Nodes and Glyphs](systems/glyphs.md) |
| Equipment Nodes and weapon/armor Sigils | [Equipment Sigils](systems/sigils.md) |
| Equipment slots, trinkets, and legendary weapons | [Equipment](systems/equipment.md) |
| Inventory capacity philosophy | [Inventory](systems/inventory.md) |
| Crafting, forging, and learned templates | [Crafting](systems/crafting.md) |
| Currency, rewards, and purchases | [Economy](systems/economy.md) |
| Gathering ore and tools | [Mining](systems/mining.md) |
| Ingredients and generated potions | [Alchemy](systems/alchemy.md) |
| Fishing tools, activities, and rewards | [Fishing](systems/fishing.md) |
| Companions and mounts | [Companions](systems/companions.md) |
| Churches and Goddess statues | [Fast travel](systems/fast-travel.md) |
| Experience, levels, build progression, and natural restrictions | [Progression](systems/progression.md) |
| Fighting Styles, martial techniques, and mastery | [Fighting Styles](systems/fighting-styles.md) |
| Elemental Trials, unlocks, and awakening | [Elemental Trials](systems/elemental-trials.md) |
| Trial of Cataclysm and resource persistence | [Cataclysm](systems/cataclysm.md) |
| Arena participation and magic restrictions | [Arena](systems/arena.md) |
| Knight Shrines and The Kings | [Shrines](systems/shrines.md) |
| Abyss structure and progression | [The Abyss](systems/abyss.md) |
| Honor of the King | [Honor of the King](systems/honor-of-the-king.md) |
| Distinct endgame activities | [Endgame design](systems/endgame.md) |

## World

The [world index](world/README.md) covers world structure, regions, progression and difficulty, exploration and travel, transportation, hidden routes, environmental systems, encounters, regional resources, saved state, terminal presentation, logging, data, and modding.

Read [basic lore](world/lore.md) for the provisional failed-summoning premise. Character-creation rules shared by the former world and systems drafts are consolidated in the gameplay document above.

The [NPC documentation area](npc/README.md) is reserved for dedicated character and faction specifications when those are defined.

## Development and Migration

- [Planning inventory](dev/planning-inventory.md): catalog of preserved systems, world material, repository assets, and open decisions for the next planning stage.
- [System design principles](dev/system-design.md): configuration, modding, persistence, logging, separation, testing, and implementation order.
- [Repository workflow](dev/repository-workflow.md): repository roles, documentation CI, and future Windows/Linux build requirements.
- [Systems migration record](dev/systems-migration.md): source sections and line ranges for the former root `SYSTEMS.md`.
- [World migration record](dev/world-migration.md): source sections and line ranges for the former root `WORLDS.MD`.
- [Glyph migration record](dev/glyph-migration.md): translation and consolidation of the former `docs/GLYPH.md`.

The migration records preserve provenance for the old monolithic files. Shared status definitions live in `systems/effects.md`; shared lore lives in `world/lore.md`. The folder previously named `systens` is standardized as `systems`.

Known unresolved questions include Spirit versus Intelligence, contradictory Curse wording, potion Duration versus status Counter, some combat timing and effect decay rules, and undefined Glyph budgets. The relevant documents retain the original proposals and explain the gaps.
