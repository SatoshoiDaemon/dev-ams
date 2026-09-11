# Regions and Build Identities

## Tidal Town

ID concept:

```text
base:tidal_town
```

Tidal Town is the initial city.

It is located near the sea and acts as the player's primary introduction to the world.

Its role is to establish:

```text
basic NPC interaction
shops
equipment
early quests
crafting
combat preparation
world travel
the surrounding region
```

Tidal Town should feel relatively safe compared to the wilderness.

It is not required to remain permanently safe if campaign events change the region.

Because it is coastal, it is also a natural point for future sea travel.

Tidal Town should eventually provide access to ships capable of reaching Riptide.

The city should function as an early hub without becoming the only meaningful hub in the game.

## The Caves

ID concept:

```text
base:the_caves
```

The Caves are an early-game farming and resource region.

Primary identity:

```text
mining
basic enemies
early resources
repeatable farming
underground exploration
```

The Caves should contain substantial quantities of basic ores and materials.

They are one of the first places where players can deliberately farm resources rather than relying primarily on quest rewards.

Typical content may include:

```text
basic ores
stone
crystals
early crafting materials
common monsters
underground creatures
small treasure deposits
```

The Caves should initially be relatively manageable.

This does not mean every cave branch must be safe.

Deeper paths may contain stronger enemies, rare ore, hidden chambers, or routes to later regions.

The region should reward players who explore further than necessary.

## The Fairy Forest

Canonical display name:

```text
The Fairy Forest
```

ID concept:

```text
base:fairy_forest
```

The Fairy Forest is intentionally difficult to access.

Most obvious paths into the forest are blocked.

The primary entrance is hidden.

Discovering the entrance is part of the region's identity.

The game should not simply expose the forest as an ordinary destination from the beginning.

Possible discovery mechanisms include:

```text
exploration
NPC information
quest information
environment clues
secret paths
special events
```

The exact method may vary.

At the beginning of the campaign:

```text
Fairies = Hostile
```

After sufficient campaign progression, this state changes.

Later:

```text
Fairies = Friendly / Non-hostile
```

This must be represented through world/campaign state rather than by permanently replacing the region.

The same Fairy Forest should change according to campaign progression.

Possible state:

```text
fairy_forest.faction_state = hostile
```

Later:

```text
fairy_forest.faction_state = friendly
```

This state change may affect:

```text
enemy encounters
NPCs
dialogue
shops
quests
loot access
safe areas
travel
hidden content
```

The forest should demonstrate that world regions are capable of changing meaningfully over time.

## The Golden Desert

ID concept:

```text
base:golden_desert
```

The Golden Desert is a high-risk, high-reward exploration region.

Primary identity:

```text
rare creatures
rare minerals
caves
dangerous enemies
dragons
thieves
```

The desert should contain valuable natural resources unavailable or uncommon in earlier regions.

However, obtaining them exposes the player to substantially greater danger.

Typical threats include:

```text
desert monsters
bandits
thieves
dragons
hazardous caves
environmental exposure
```

Rare creatures should make the region valuable for specialized loot and crafting.

The Golden Desert should create tension between:

```text
resource value
exploration depth
survival risk
```

Some valuable material deposits may deliberately be located close to extremely dangerous enemy territories.

The player should be allowed to attempt these areas earlier than recommended.

## Crimson Forest

Canonical display name:

```text
Crimson Forest
```

ID concept:

```text
base:crimson_forest
```

The Crimson Forest is a substantially more dangerous forest region.

Its gameplay identity is strongly associated with:

```text
debuffs
blood
dark magic
status effects
curses
damage-over-time builds
```

Enemies are generally more dangerous than those found in ordinary early regions.

In return, the region offers particularly valuable rewards for builds centered on:

```text
debuff application
debuff potency
debuff duration
blood mechanics
dark magic
status effects
damage over time
```

Possible rewards include:

```text
rare Nodes
unique Glyphs
status-oriented equipment
blood-related materials
dark magic resources
special enemy drops
```

The Crimson Forest should not merely have stronger statistical versions of ordinary enemies.

Its enemies should make heavier use of the systems the region represents.

Examples include enemies capable of:

```text
applying bleeding
applying curses
reducing attributes
stacking status pressure
draining resources
interacting with existing debuffs
```

This region should be one of the primary locations for players intentionally building around status mechanics.

## The Ice Mountain

ID concept:

```text
base:ice_mountain
```

The Ice Mountain is dangerous both because of its enemies and because of the environment itself.

The climate is hostile.

Environmental survival is part of the region's gameplay.

Potential hazards include:

```text
extreme cold
storms
reduced visibility
slippery terrain
exposure
resource drain
restricted movement
```

The environment should remain relevant even if no enemy is currently present.

The mountain contains extremely dangerous monsters.

Its enemies should represent a significant increase in threat compared to early regions.

Preparation should matter.

Possible preparation may include:

```text
appropriate equipment
cold resistance
consumables
specific magic
camping resources
specialized builds
```

Do not make the environment a simple binary requirement where the player either has one item or instantly dies.

Prefer escalating pressure and systems the player can interact with.

## Apexia

ID concept:

```text
base:apexia
```

Apexia is a technologically advanced city with a steampunk-inspired identity.

Its defining characteristic is the combination of:

```text
technology
engineering
machinery
magic
weapons
```

Apexia has substantially more advanced technology than most other known regions.

Magic and technology are not treated as mutually exclusive.

Instead, the city actively combines them.

Apexia is the primary destination for players interested in weapon-focused builds.

Especially:

```text
firearms
mechanical weapons
advanced ranged weapons
weapon modifications
engineered equipment
magic-tech hybrids
```

A player pursuing a weapon-centric build should strongly benefit from reaching Apexia.

Possible services include:

```text
gunsmiths
engineers
weapon modification
rare ammunition
mechanical crafting
advanced components
specialized merchants
```

The city should have its own distinct culture, architecture, economy, and enemy ecosystem.

Do not reduce Apexia to simply "the gun shop city."

Weapon specialization is one of its major mechanical roles, but the region should remain a complete location.

## Riptide

ID concept:

```text
base:riptide
```

Riptide is located across the ocean.

Normal land travel cannot reach it.

The player reaches Riptide by ship.

This introduces a distinct travel requirement:

```text
Sea Route
```

Travel should originate from an appropriate port, initially most naturally Tidal Town or another coastal location.

Riptide provides access to stronger or otherwise superior items compared to early regions.

However, the city is dominated by pirates.

Pirates are not merely random enemies outside the city.

They have substantial political and social control over the region.

This should influence:

```text
NPCs
law
shops
quests
factions
economy
crime
travel
```

Riptide contains bounty-hunting missions focused on pirates.

The player should be able to accept contracts to hunt specific targets.

Possible bounty structure:

```text
Target
Difficulty
Known Location
Faction
Reward
Special Conditions
```

Higher-value pirate targets should have stronger crews, better equipment, more dangerous locations, or unique mechanics.

Bounty hunting should be one of Riptide's major repeatable progression systems.

## Regional Build Identity

Major regions should intentionally support different build archetypes.

Current core associations include:

```text
The Caves
→ mining, basic crafting, early farming

The Fairy Forest
→ fairy/magical content, campaign-dependent faction content

The Golden Desert
→ rare resources, dangerous creatures, dragons

Crimson Forest
→ debuffs, blood, dark magic, status builds

The Ice Mountain
→ survival, hostile environment, dangerous monsters

Apexia
→ weapons, firearms, engineering, technology + magic

Riptide
→ improved items, pirates, bounty hunting
```

These are identities, not exclusive restrictions.

For example, dark-magic equipment may exist outside the Crimson Forest.

The Crimson Forest should simply be one of the strongest sources for that style of progression.
