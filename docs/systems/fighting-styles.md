# Fighting Styles and Martial Techniques

> **Specification status:** The canonical section defines mastery, slots, Node budgets, unlocks, and cross-influence values. Fighting Styles remain passive frameworks; active techniques remain equipment/weapon actions.

See [Scaling](scaling.md) for shared attribute scaling rules and [combat](combat.md) for action ordering and equipment use.

## Fighting Styles

Fighting Styles are passive combat frameworks.

The player can carry/equip up to:

2 Fighting Styles

at the same time.

Fighting Styles are built from Nodes.

They do NOT normally provide active combat abilities themselves.

Their role is to modify the way the character fights.

## Starting Fighting Styles

The player begins with:

- Basic Closed Combat
- Basic Weapon Combat

These are the foundational Fighting Styles.

They should remain useful as introductory frameworks while being simpler than later styles.

Both have their own Node structures.

## Unlocking Fighting Styles

Additional Fighting Styles are discovered through the world.

Sources include:

- exploration
- quests
- enemies
- masters
- dojos
- swordsmen
- special NPCs
- dungeons
- rare techniques

Examples:

```text
Dojo Master
→ teaches unarmed style.
```

```text
Swordsman
→ teaches sword-oriented style.
```

Styles should feel like learned combat disciplines rather than ordinary loot stat bonuses.

## Fighting Style Nodes

Fighting Styles are constructed using Nodes.

Their Node pools are distinct from ordinary spell/equipment Node pools where appropriate.

Fighting Style Nodes may modify:

- critical damage
- vampirism
- armor penetration
- Scaling
- attack properties
- weapon interactions
- ranged interactions
- catalyst interactions
- magic interactions
- unarmed combat

The exact style defines which modifications are available.

## Passive Nature of Fighting Styles

Fighting Styles do NOT provide ordinary active skills by default.

This is intentional.

A Fighting Style changes:

how you fight

rather than adding another action bar.

For active pure-melee techniques, the player uses an appropriate weapon/equipment type.

## Unarmed Technique Weapons

Active unarmed techniques are represented through dedicated weapon types.

Example:

Caestus

These weapons mechanically represent:

- fists
- gauntlets
- martial techniques
- unarmed combat frameworks

This allows active martial abilities to participate in the same:

- weapon
- Node
- Scaling
- equipment
- combat

systems as other offensive tools.

The Fighting Style itself remains passive.

## Basic Style Node Distinction

Basic Closed Combat and Basic Weapon Combat should possess Node structures that are distinct from later specialized Fighting Styles.

They are foundational styles, not merely weaker copies of advanced styles.

The exact Node sets are defined elsewhere.

## Fighting Style Scaling

Fighting Styles may possess Scaling.

Unlike static weapon Scaling, Fighting Style Scaling may improve through:

- Nodes
- style usage
- mastery
- progression

The original draft did not define a mastery formula. The canonical mastery formula is defined in [Canonical Quantified Fighting Style Contract](#canonical-quantified-fighting-style-contract).

Do not invent an arbitrary usage grind formula without explicit design approval.

## Fighting Style Cross-Influence

Fighting Styles are primarily oriented toward close combat, but they may influence:

- melee weapons
- ranged weapons
- catalysts
- magic

depending on the style.

This permits unusual hybrid builds.

However, pure martial combat should receive the strongest and most common Fighting Style support.

## Canonical Quantified Fighting Style Contract

The player may equip exactly 2 Styles. Each equipped Style has Mastery `0..100`; Mastery is stored per stable style ID. A successful use that resolves an attack or defensive event grants `1 Mastery XP`, a boss victory grants `10`, and a failed/missed action grants `0`; XP required for the next Mastery point is `100`, so one normal successful use advances `1%`. Mastery cannot be gained more than once per actor turn.

Style tiers unlock at Mastery `0, 25, 50, 75, 100`. A Style has `10` Node Points at Mastery 0 and gains `1` at each tier, for `14` total. A Node costs `1..4` points; the style cannot exceed its points. Node weight is included in the style's build budget, not spell Weight. Each Style has budget `10 + floor(Mastery / 10)` Node Capacity; a selected Node's cost must fit this capacity.

Basic Closed Combat grants the following fixed nodes: Unarmed Damage `+10%`, Evasion `+5%`, and Stamina Cost `-10%`. Basic Weapon Combat grants Weapon Damage `+10%`, Weapon Scaling `+5%`, and Swap Action Cost `-25%`. Each is available at Mastery 0 and costs 1 Node Point per node.

Specialized styles use these numeric Node classes: Damage/Defense/Scaling modifiers cost 2 points for `+5%`; critical modifiers cost 3 points for `+5 percentage points`; status application modifiers cost 2 points for `+10% Potency and Counter`; armor penetration costs 2 points per `+5`; resource modifiers cost 2 points per `+5%`; cross-system modifiers cost 4 points per `+10%`. No style modifier may exceed `+50%` without a named legendary Node.

Cross-influence is explicit: a melee style affects melee/body weapons at `100%`, ranged weapons at `25%`, catalysts/spells at `10%`; a ranged style affects ranged at `100%`, melee at `25%`, magic at `25%`; a magic style affects catalysts/spells at `100%`, weapons at `10%`; a universal style affects all at `50%`. The style definition chooses one profile. Modifiers apply in the Equipment/Active Effects stages and are visible in `explain last`.

Unlocks require an explicit source and thresholds: starting styles are free; a discovered style requires its stable unlock record plus either a teacher quest or `1000 Gold`; legendary styles require Mastery 75 in one prerequisite style. Respec costs `100 × current level` Gold and cannot delete a learned style. Tests must cover mastery gain caps, Node budget overflow, dual-style stacking, cross-influence coefficients, and status Glyph amplification from style Nodes.
