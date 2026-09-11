# World Documentation Migration

All design content from the former root `WORLDS.MD` is preserved in the dedicated English documents below. The source was already in English; its requirements, examples, IDs, numeric values, and tentative language were retained. Section numbering was replaced with descriptive Markdown headings.

Source snapshot: 1,411 lines; SHA-256 `627413E5C9AA5470668DC06663AD2A46BFE485E228B7CCACE13EE580D1292270`.

Line numbers refer to that source snapshot, before removal. They are provenance references, not links to an active source document.

| Original section | Original lines | Current document |
| --- | --- | --- |
| Document title and spacing | 1–2 | [World documentation index](../world/README.md) |
| 1. World Structure | 3–45 | [World structure and design](../world/overview.md) |
| 2. Region Design Philosophy | 46–75 | [World structure and design](../world/overview.md) |
| 3. World Data | 76–132 | [World data and modding](../world/data-and-modding.md) |
| 4. Progression Model | 133–177 | [World progression and difficulty](../world/progression.md) |
| 5. Open World | 178–224 | [Travel](../world/travel.md) |
| 6. Region Connections | 225–271 | [Travel](../world/travel.md) |
| 7. Tidal Town | 272–308 | [Regions](../world/regions.md#tidal-town) |
| 8. The Caves | 309–354 | [Regions](../world/regions.md#the-caves) |
| 9. The Fairy Forest, before the inserted lore block | 355–401 | [Regions](../world/regions.md#the-fairy-forest) |
| Inserted block separator and heading | 402–405 | Editorial relocation described below |
| Basic Lore | 406–432 | [Basic lore](../world/lore.md) |
| Character Creation | 433–444 | [Character creation](../systems/character-creation.md) |
| Races | 445–475 | [Character creation](../systems/character-creation.md) |
| Initial Elements | 476–504 | [Character creation](../systems/character-creation.md) |
| Element Ownership | 505–522 | [Character creation](../systems/character-creation.md) |
| Copied-source provenance note and separators | 523–526 | Editorial relocation described below |
| 9. The Fairy Forest, friendly-state continuation | 527–564 | [Regions](../world/regions.md#the-fairy-forest) |
| 10. The Golden Desert | 565–616 | [Regions](../world/regions.md#the-golden-desert) |
| 11. Crimson Forest | 617–687 | [Regions](../world/regions.md#crimson-forest) |
| 12. The Ice Mountain | 688–738 | [Regions](../world/regions.md#the-ice-mountain) |
| 13. Apexia | 739–799 | [Regions](../world/regions.md#apexia) |
| 14. Riptide | 800–863 | [Regions](../world/regions.md#riptide) |
| 15. Regional Build Identity | 864–900 | [Regions](../world/regions.md#regional-build-identity) |
| 16. World State | 901–934 | [World state and persistence](../world/state-and-persistence.md) |
| 17. Hidden Locations and Routes | 935–977 | [Hidden locations and routes](../world/hidden-routes.md) |
| 18. Transportation | 978–1017 | [Transportation](../world/transportation.md) |
| 19. Environmental Systems | 1018–1058 | [Environmental systems](../world/environment.md) |
| 20. Encounter Tables | 1059–1096 | [Encounter tables](../world/encounters.md) |
| 21. Resource Distribution | 1097–1130 | [Resource distribution](../world/resources.md) |
| 22. Regional Difficulty | 1131–1154 | [World progression and difficulty](../world/progression.md) |
| 23. Campaign Changes | 1155–1191 | [World state and persistence](../world/state-and-persistence.md) |
| 24. Terminal Region Representation | 1192–1227 | [Terminal representation](../world/terminal-rendering.md) |
| 25. World Logging | 1228–1262 | [World logging](../world/logging.md) |
| 26. Base World Topology | 1263–1301 | [World structure and design](../world/overview.md) |
| 27. Known Regions | 1302–1323 | [World structure and design](../world/overview.md) |
| 28. World Modding | 1324–1364 | [World data and modding](../world/data-and-modding.md) |
| 29. Save Compatibility | 1365–1402 | [World state and persistence](../world/state-and-persistence.md) |
| 30. World System Rule | 1403–1411 | [World structure and design](../world/overview.md) |

## Editorial Relocation

The source inserted a lore and character-creation block between the Fairy Forest's `Later:` sentence and its `Fairies = Friendly / Non-hostile` example. The two parts of the Fairy Forest description are now adjacent in the regional document, preserving both hostile and friendly campaign states.

The inserted block stated that it had been copied from `SYSTEMS.md`, which remained authoritative at that time. Its original provenance is retained here. After migration, [basic lore](../world/lore.md) is the canonical home of that shared lore, and [character creation](../systems/character-creation.md) is the canonical home of the shared character mechanics. The old note's statement that the root systems file remains authoritative is superseded by this dedicated documentation structure.

The source's tentative topology, conceptual region IDs, illustrative JSON and Lua API, possible hazards and services, configurable environment example (`cold_intensity = 0.8`), and illustrative level-30 restriction example are all retained without becoming new fixed gameplay rules.

## Coverage Verification

All 902 nonempty source content lines were checked against the world documents and the shared character-creation document, with no missing lines. This comparison excludes headings, horizontal separators, and the superseded provenance note explained above; it normalizes Markdown bullet prefixes added to character-creation lists. All 57 local document links in this world documentation and migration record resolve, and all fenced code blocks are balanced.
