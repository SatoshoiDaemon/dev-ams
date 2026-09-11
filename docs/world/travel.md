# Open World Travel and Region Connections

## Open World

The Open World represents the territory between major handcrafted regions.

It acts as the main connective tissue of the world.

The player physically travels through it using the normal terminal exploration system.

The Open World may contain:

```text
roads
small encounters
resource nodes
temporary camps
wandering enemies
merchants
bandits
minor caves
ruins
random events
weather
hidden paths
```

It is not merely a menu.

The travel menu appears only when the player reaches an appropriate boundary or transition point.

At that point, only nearby/connected destinations are presented.

Example:

```text
You reached the northern road.

Where do you want to travel?

> The Caves
> The Fairy Forest
> Return to Tidal Town
```

A destination must have an actual valid route from the current region.

## Region Connections

Region connectivity should be explicit.

Each connection may define properties such as:

```text
source
destination
bidirectional
hidden
locked
transport type
campaign requirement
item requirement
faction requirement
environment requirement
```

A connection does not have to be permanently available.

Examples:

```text
road
mountain pass
cave tunnel
hidden forest entrance
ship route
collapsed passage
portal
```

Connections should be capable of changing over the campaign.

For example:

```text
a bridge can be destroyed
a pirate route can open
a hidden forest entrance can be discovered
a cave can collapse
a faction can grant access
```
