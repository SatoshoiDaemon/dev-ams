# Crafting, Forging, and Templates

## Crafting and Forging

The player can craft equipment.

Primary crafted equipment includes:

- weapons
- armor
- possibly catalysts
- other equipment

The player begins knowing a limited set of simple recipes.

Better recipes are discovered through:

Blacksmith Templates

Templates may be obtained through:

- exploration
- treasure
- fishing
- quests
- enemies
- shops
- special regions
- dungeons

Once a valid template is learned, the character gains access to its corresponding crafting recipe or crafting structure.

Crafting should interact with:

- materials
- weapon templates
- equipment Nodes
- scaling
- regional resources

## Canonical Recipe Contract

Crafting is a direct recipe operation, not a generic quality or failure simulation. A recipe has a stable ID, input item IDs and quantities, and output item IDs and quantities.

```text
base:arrow:
    inputs:
        base:wood: 1
        base:stone: 1
    output:
        base:arrow: 1
```

When all inputs are present and the recipe is known, crafting consumes the declared inputs and creates the declared outputs. There is no generic quality, budget, failure, waste, durability, or material-grade rule. A recipe may declare a specific exception, but it must not be inferred by the engine.

Invalid recipe data is rejected with the recipe ID, file, field, and reason. Learned templates remain stable IDs in character/save progression; owning a physical template item is not required after learning it.

## Crafting Templates

Templates represent discovered construction knowledge.

Templates should use stable IDs.

Example concept:

- base:iron_greatsword_template
- base:revolver_template
- base:dragon_plate_template

Templates belong to character/save progression.

A template should not need to remain physically in the inventory forever after being learned unless a specific crafting mechanic requires it.
