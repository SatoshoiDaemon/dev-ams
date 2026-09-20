# Companions and Mounts

## Companions and Mounts

The default player companion capacity is:

- 1 Mount
- 2 Pets

Mounts primarily improve world traversal speed.

Pets may provide combat or passive utility depending on their definition. The player selects the actions of companions that participate in combat; companions do not make autonomous combat decisions through an AI system.

The default limits are configurable and may be increased through:

- equipment
- armor
- passives
- configuration
- mods

A companion with active combat actions receives its own turn, and the player supplies its action. A passive-only companion does not receive an independent turn.

## Canonical Companion and Mount Rules

When a monster is defeated, each eligible companion receives `60%` of that monster's awarded experience. Companion defeat is temporary: a defeated companion returns after fast travel, a rest, or the player's death.

Mount traversal depends on movement capabilities:

- a land mount traverses land;
- a mount that can swim ignores water traversal restrictions;
- a mount that can fly ignores both land and water traversal restrictions.

Companion state, ownership, defeat, return, and experience must be represented by stable IDs in the save. Mount traversal is an exploration rule and does not create combat AI.

A companion that only provides passive effects does not receive an independent turn.
