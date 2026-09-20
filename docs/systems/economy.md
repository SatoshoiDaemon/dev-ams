# Economy

## Economy

The game uses a simple local in-world currency system. It is not an economic simulation.

The player gains money primarily through:

- defeating enemies
- selling items
- completing certain quests
- location-specific activities
- bounties

Not every enemy necessarily needs to literally carry currency.

Reward sources may be contextual.

Money can be spent on things such as:

- weapons
- armor
- items
- consumables
- materials
- services
- templates
- fishing equipment
- crafting resources

The economy exists primarily as another progression mechanism.

Currency values, merchant multipliers, rewards, prices, and loot economy should be configurable where practical.

## Canonical Economy Contract

Currency is an integer player-owned resource. Systems may award or remove declared amounts, and items or services may define declared prices. Purchases fail without changing state when the player lacks sufficient currency. Selling, merchants, dynamic stock, regional pricing, inflation, and buyback rules exist only if a specific content definition adds them; there is no generic economy model that requires them.

Currency changes should identify the source, reason, amount, and resulting balance in player-readable logs.
