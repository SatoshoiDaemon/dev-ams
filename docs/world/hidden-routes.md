# Hidden Locations and Routes

The world must support hidden destinations and connections as a first-class mechanic.

The Fairy Forest is an explicit initial example.

A hidden route may exist physically while remaining unavailable in normal destination menus until discovered.

Conceptually:

```text
Connection exists:
Tidal Region → Fairy Forest

discovered = false
```

After discovery:

```text
discovered = true
```

Discovery should normally persist in the save.

Hidden routes may be discovered through:

```text
movement
interaction
quest events
items
NPC dialogue
perception-like mechanics
Lua scripts
```

The world engine should not hardcode hidden-route behavior specifically for the Fairy Forest.

It should be a generic system usable by mods and future regions.
