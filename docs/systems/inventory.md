# Inventory

## Inventory Philosophy

Inventory has no gameplay capacity restriction. It has no categories, weight, capacity, or maximum stack size, and items do not become invalid because the player owns many of them.

Chests are organizational storage. Moving an item to a chest changes its location/ownership record; it does not solve an inventory capacity problem and does not create a separate gameplay limitation.

The technical contract is limited to item identity, quantity when the item is countable, current location, and save serialization. Equipment slots and quick-access selections are separate systems and do not define inventory capacity.
