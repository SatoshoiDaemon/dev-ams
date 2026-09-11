# Systems Documentation Migration

The former root `SYSTEMS.md` contained 3,834 lines. Its file-label line is represented by this provenance record; every substantive line is assigned below. Original section numbers are retained only in this map because the source restarted numbering at Mining after inserting a large Status Effect specification. Destination headings omit those conflicting numbers.

The migration preserves rules, examples, formulas, limits, tentative language, and open questions. Plain lists and diagrams have been given Markdown formatting; fenced-block editor IDs have been removed. This is a documentation migration, not a gameplay implementation or a decision to resolve competing drafts.

| Original lines | Original section(s) | Destination |
| --- | --- | --- |
| 1 | Source file label | This provenance record |
| 21–47 | 2. Basic Lore | [World lore](../world/lore.md) |
| 2–20 | 1. Purpose | [System Design Principles](../dev/system-design.md) |
| 3521–3632 | 91. Configuration Requirements through 94. Logging Requirements | [System Design Principles](../dev/system-design.md) |
| 3679–3834 | 96. System Separation through 99. Core Systems Rule | [System Design Principles](../dev/system-design.md) |
| 48–137 | 3. Character Creation through 6. Element Ownership | [Character Creation, Races, and Element Ownership](../systems/character-creation.md) |
| 138–169 | 7. Economy | [Economy](../systems/economy.md) |
| 170–195 | 8. Critical Damage | [Combat](../systems/combat.md) |
| 2290–2600 | 24. Combat Entry through 38. End-of-Round Processing | [Combat](../systems/combat.md) |
| 3633–3678 | 95. Combat Explainability | [Combat](../systems/combat.md) |
| 196–1972 | 9. Status Effect System through 45. Status System Rule | [Status Effects](../systems/effects.md) |
| 1973–2008 | 11. Mining | [Mining](../systems/mining.md) |
| 2009–2061 | 12. Crafting and Forging through 13. Crafting Templates | [Crafting, Forging, and Templates](../systems/crafting.md) |
| 2062–2082 | 14. Inventory Philosophy | [Inventory](../systems/inventory.md) |
| 2083–2124 | 15. Armor Slots through 16. Trinkets and Talismans | [Equipment and Legendary Weapons](../systems/equipment.md) |
| 3383–3411 | 85. Excalibur through 86. Demon King's Sword | [Equipment and Legendary Weapons](../systems/equipment.md) |
| 2125–2143 | 17. Fast Travel | [Fast Travel](../systems/fast-travel.md) |
| 2144–2166 | 18. Companions and Mounts | [Companions and Mounts](../systems/companions.md) |
| 2167–2235 | 19. Alchemy through 21. Dynamic Potions | [Alchemy and Dynamic Potions](../systems/alchemy.md) |
| 2236–2289 | 22. Fishing through 23. Fishing Rewards | [Fishing](../systems/fishing.md) |
| 2601–2681 | 39. Experience and Levels through 42. Hard Limits | [Levels and Build Progression](../systems/progression.md) |
| 3441–3520 | 88. Progression Sources through 90. Natural Restrictions | [Levels and Build Progression](../systems/progression.md) |
| 2682–2829 | 43. Core Attributes through 51. HP and Tenacity | [Attributes, Power, and Defensive Resources](../systems/attributes.md) |
| 2830–2954 | 52. Fighting Styles through 58. Basic Style Node Distinction | [Fighting Styles and Martial Techniques](../systems/fighting-styles.md) |
| 3051–3080 | 65. Fighting Style Scaling through 66. Fighting Style Cross-Influence | [Fighting Styles and Martial Techniques](../systems/fighting-styles.md) |
| 2955–3050 | 59. Scaling System through 64. Art of War Scaling | [Attribute Scaling](../systems/scaling.md) |
| 3081–3138 | 67. Elemental Trials through 70. Awakened Elements | [Elemental Trials and Awakening](../systems/elemental-trials.md) |
| 3139–3175 | 71. Trial of Cataclysm through 72. Cataclysm Resource Persistence | [Trial of Cataclysm](../systems/cataclysm.md) |
| 3176–3226 | 73. The Arena through 75. Arena Magic Restriction | [The Arena](../systems/arena.md) |
| 3227–3289 | 76. The Shrines through 79. The Kings | [Knight Shrines and The Kings](../systems/shrines.md) |
| 3290–3369 | 80. Abyss through 83. Abyss Progression | [The Abyss](../systems/abyss.md) |
| 3370–3382 | 84. Honor of the King | [Honor of the King](../systems/honor-of-the-king.md) |
| 3412–3440 | 87. Endgame Philosophy | [Endgame Activity Design](../systems/endgame.md) |

## Consolidated Repetitions

- The status material copied into the Glyph notes resolves to [Status Effects](../systems/effects.md). Fear, Taunt, and Charm retain their source-entity semantics. All 25 base effects, the canonical table, and the behavior summary remain in that dedicated document.
- The lore and character-creation material copied into `WORLDS.MD` resolves to [World lore](../world/lore.md) and [Character Creation](../systems/character-creation.md). The initial race list, up to two selected elements, all nine options, later unlocks, and persisted element states remain intact.
- Related source sections are grouped by subsystem: for example crafting and templates, alchemy and potion generation, combat and its Priority log, and Fighting Styles and their Scaling progression.
- The Portuguese localization list for the seven core attributes is translated to the English names, following the requested documentation-language standard. It is kept as a second standardized list rather than deleting the source's localization reference.

## Preserved Open Decisions and Conflicts

- **Spirit / Intelligence:** the systems draft says Spirit supersedes Intelligence, while repository instructions define Intelligence and give it additional Node/complexity responsibilities. [Attributes](../systems/attributes.md) preserves the draft and calls out the unresolved conflict.
- **Curse:** the source simultaneously increases and decreases other Debuffs' efficiency by 1% per Counter. No replacement target/category is invented; the confirmed `Counter -= 1 per turn` remains.
- **Potion Duration / Status Counter:** potion properties include Intensity, Potency, and Duration; the Status Effect schema prohibits an independent generic Duration. [Alchemy](../systems/alchemy.md) marks the conversion/terminology decision as unresolved.
- **Combat timing:** the player's first action opportunity is guaranteed, but the opening condition is not fully defined; Bonus Action recursion limits and fleeing formulas remain unspecified.
- **Incomplete effect details:** unspecified decay rules, Bleed's damage type, and Poison continuity after replacement are not filled in by assumption.
- **Scaling:** ordinary grades D–S and possible exceptional grades SS/SSS are both preserved; this migration does not invent new multipliers or a Fighting Style mastery formula.

The source's default values remain subject to its configuration and modding requirements. Provisional race mechanics, equipment categories, Trial persistence details, legendary properties defined elsewhere, and all other tentative statements remain tentative.

## Migration Verification

A comparison of normalized source and destination lines verified all 2,121 nonempty content occurrences assigned to the 24 documents generated from this source. Normalization accounted only for Markdown formatting, heading numbers, and the explicitly translated localization labels. The 27-line lore section was independently checked against its identical copy in the world draft and consolidated into world lore. All source lines have a destination; all destination code fences are balanced and relative links resolve.
