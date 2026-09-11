# Mining

## Mining

Mining is a resource-gathering subsystem.

The player requires a pickaxe.

Mining occurs primarily in caves and other mineral-rich regions.

Basic loop:

```text
Find mineral deposit
↓
Possess appropriate pickaxe
↓
Mine
↓
Receive ore/material
↓
Sell or craft with it
```

Mining tools do NOT use durability.

There may be material/tool requirements.

For example:

```text
Basic Pickaxe
→ common ores
```

```text
Advanced Pickaxe
→ harder or rarer ores
```

The exact progression is configurable and may differ between resources.

Do not implement durability unless that design decision changes explicitly.
