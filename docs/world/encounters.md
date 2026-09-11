# Regional Encounter Tables

Each region should have configurable encounter tables.

Encounter definitions should be capable of considering:

```text
region
sub-region
time/state
campaign progress
weather
player actions
faction state
rarity
```

Example:

```text
Golden Desert
    common:
        desert_bandit
        dune_beast

    rare:
        golden_scorpion

    very_rare:
        desert_dragon
```

Exact enemies are not defined by this document.

Mods should be capable of adding entries without rewriting the entire region.
