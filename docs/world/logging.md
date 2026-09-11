# World Logging

World travel and loading should integrate with the project's logging system.

Useful logs include:

```text
[INFO] Loading region: base:golden_desert
[INFO] Leaving region: base:tidal_town
[INFO] Route selected: tidal_coast -> golden_desert
[INFO] Hidden route discovered: fairy_forest_west_entrance
```

Failures should provide context.

Example:

```text
[ERROR] Failed to enter region.

Region:
base:fairy_forest

Route:
base:hidden_fairy_entrance

Reason:
Referenced region data could not be loaded.

Source:
mods/example_world/regions/fairy_forest.json
```
