# Glyph Design Migration Record

This record maps all 753 lines of the former `docs/GLYPH.md` design draft to dedicated English documentation. The source originally mixed Portuguese proposals with an English excerpt copied from `SYSTEMS.md`. Blank separators are included in the adjacent ranges below; no gameplay content is omitted.

| Original source lines | Original subject | Destination |
| --- | --- | --- |
| 1–75 | General magic Node grammar: Source, Type, Form, Effect, Modifier, Sacrifice | [General Magic Node Grammar](../systems/glyphs.md#general-magic-node-grammar) |
| 76–148 | Element-independent Glyphs; Counter/Potency examples; eight specialized status Glyphs; fixed Stun editor values | [General Status Glyphs](../systems/glyphs.md#general-status-glyphs) |
| 149–162 | Element-exclusive introduction; Fire identity and five Glyphs | [Fire](../systems/glyphs.md#fire--combustion-and-consumption) |
| 163–174 | Water identity, five Glyphs, and `10 / 10 / 70 → 30 / 30 / 30` example | [Water](../systems/glyphs.md#water--flow-and-redistribution) |
| 175–186 | Earth identity and five Glyphs | [Earth](../systems/glyphs.md#earth--stability-tenacity-and-impact) |
| 187–198 | Air identity and five Glyphs | [Air](../systems/glyphs.md#air--movement-priority-and-trajectory) |
| 199–207 | Light identity and five Glyphs | [Light](../systems/glyphs.md#light--purification-revelation-and-protection) |
| 208–216 | Darkness identity and five Glyphs | [Darkness](../systems/glyphs.md#darkness--appropriation-corruption-and-debt) |
| 217–228 | Poison identity, five Glyphs, and many-debuff build example | [Poison](../systems/glyphs.md#poison--deterioration-and-debuff-synergy) |
| 229–242 | Ice identity and five Glyphs; distinction from Water | [Ice](../systems/glyphs.md#ice--preservation-and-breaking) |
| 243–254 | Sound identity and five Glyphs | [Sound](../systems/glyphs.md#sound--resonance-repetition-and-interruption) |
| 255–274 | Metal identity, five Glyphs, and Forge Bond spell tree | [Metal](../systems/glyphs.md#metal--armor-penetration-and-equipment) |
| 275–286 | Plant identity and five Glyphs | [Plant](../systems/glyphs.md#plant--growth-and-persistence) |
| 287–307 | Lava identity, five Glyphs, and 17-turn Eruption example | [Lava](../systems/glyphs.md#lava--territory-and-inevitability) |
| 308–316 | Lightning identity and five Glyphs | [Lightning](../systems/glyphs.md#lightning--networks-speed-and-overload) |
| 317–360 | Ten equipment triggers, ten effects, and six general Sigils | [Equipment Nodes and General Sigils](../systems/sigils.md#equipment-nodes) |
| 361–400 | Weapon-template exclusivity; nine weapon Sigils from Single Hand Light to Bow | [Weapon-Category Sigils](../systems/sigils.md#weapon-category-sigils) |
| 401–566 | Canonical Status Effect System excerpt copied from `SYSTEMS.md`, including its attribution note | [Status Effects](../systems/effects.md); canonical-source relationship retained in the opening paragraph of [Magic Nodes and Glyphs](../systems/glyphs.md) |
| 567–589 | Five remaining weapon Sigils from Crossbow to Colossal Weapon; reason for mechanical distinctions | [Weapon-Category Sigils](../systems/sigils.md#weapon-category-sigils) |
| 590–619 | Six armor-class Sigils and Robe identity | [Armor Sigils](../systems/sigils.md#armor-sigils) |
| 620–677 | Catalyst Property/Core/Glyph distinction; Staff Focus, Grimoire Inscription, Seal Imposition, numeric budget examples, Dragon Heart Core tree | [Catalyst Glyph Specializations](../systems/glyphs.md#catalyst-glyph-specializations) |
| 678–725 | Universal status editor; Weight examples; elemental branch compatibility and valid/invalid trees | [Universal Status Interface](../systems/glyphs.md#universal-status-interface) |
| 726–753 | Four Glyph design questions; `+15% damage` example; Node/Glyph/Sigil responsibilities | [Node, Glyph, and Sigil Responsibilities](../systems/glyphs.md#node-glyph-and-sigil-responsibilities) |

## Canonical Excerpt Coverage

The copied status excerpt is consolidated into `docs/systems/effects.md`, alongside its fuller original source, instead of retaining a second authority in the Glyph catalog. It covers:

- Counter as remaining lifetime and Potency as intensity; no third independent generic Duration field.
- Default Potency and Counter limits of `0–99`, with per-effect exceptions and fixed/unused values.
- The DoT, Crowd Control, Buff, and Debuff categories, inspectable by mods.
- Definition fields: ID, Name, Category, Potency Limit, Counter Limit, Fixed Potency, Uses Potency, Uses Counter, Tick Behavior, Trigger Behavior, Modifiers, and Tags; simple data definitions and embedded Lua for complex behavior.
- Competition by Potency first, then Counter; the weaker instance is discarded instead of automatically stacking.
- Per-effect event processing for Round Start, Action Declaration, Before Action, Attack, Evade, Damage, HP Change, End of Turn, End of Round, Status Applied, and Status Removed; no assumed universal decay.
- Burn: Magical Damage equal to Potency; loses one Potency and one Counter every turn.
- Poison: Magical Damage equal to Potency times continuous Turns Under Poison; tracks that duration; loses half its Counter every turn with floor rounding.
- Sinking: Energy/Mana loss equal to Potency each turn; Counter decreases by one; Potency normally does not decay.
- Electrified: Magical Damage equal to Potency times the currently Electrified targets on the battlefield; Counter decreases by one at end of turn.
- Bleed: damage equal to Potency on an attack or successful evade; loses half of Potency and Counter at each turn end, with floor rounding.
- Stun: cannot attack or evade, fixed Potency `1`, maximum Counter `5`.
- Binding: blocks movement and physical actions, may allow long-range/magic, fixed Potency `1`, maximum Counter `5`.
- Silent: prevents magic while allowing other actions.
- Fear: records source, prevents attacking the applier, and reduces Evasion by `Potency × 1%`.
- Taunt: records source, permits attacks only against the applier, and reduces damage dealt by `Potency × 1%`.
- Charm: records source, prevents attacking the applier, and increases the source's damage against the affected target by `Potency × 1%`.

## Preservation and Unresolved Details

- The translated catalogs preserve all 65 elemental Glyphs, eight general status Glyphs, six general equipment Sigils, 14 weapon-template Sigils, six armor-class Sigils, and three catalyst specializations.
- All source Node trees, enumerations, numerical examples, UI examples, and design rationales are retained in their dedicated system documents. The copied status excerpt is represented by the canonical document and the checklist above.
- Proposal language remains proposal language. Undefined balancing values, budget conversion formulas, event mappings, and graph representation are identified as open details rather than resolved by invention.
- `Duration+` remains a Node label, with an explicit note that status lifetime uses Counter. The original Staff Focus tree and the general Type grammar are both retained, and their representation mismatch is recorded in `glyphs.md`.
- Legacy file references in this record describe provenance only; current documentation links point to the dedicated Markdown files.
