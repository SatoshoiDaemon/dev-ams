# Fast Travel

Fast Travel uses the world graph only to determine which declared points exist and which destinations are available.

## Fast Travel

Fast travel is available through:

- Churches
- Statues of a Goddess

These locations act as fast-travel anchors.

When the player reaches a Fast Travel Point, that point becomes `unlocked = true` in the save. Unlocked points can be selected as destinations.

Fast Travel has no cost, cooldown, travel time, risk, or additional generic requirement. Specific campaign content may define whether a point exists or can be reached; the Fast Travel mechanic itself does not add restrictions.

Fast travel should not replace world exploration.

Its primary purpose is reducing repeated travel through already explored territory.

Campaign state may change which points exist or are unlocked, but Fast Travel adds no generic cost, cooldown, time, risk, or extra restriction.

Fast travel behavior should be data-driven enough for mods to register additional travel anchors.
