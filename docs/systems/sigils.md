# Equipment Sigils

This document preserves the initial proposals for special equipment behavior. [Magic Nodes and Glyphs](glyphs.md) documents the related magic grammar, elemental Glyphs, and catalyst specializations. The catalog is a design proposal rather than a complete set of numeric balancing formulas or event API contracts.

## Equipment Nodes

The proposed equipment Nodes remain the triggers:

```text
On Hit
On Critical
On Block
On Parry
On Dodge
On Kill
On Damage Taken
While Equipped
Below X% HP
Above X% HP
```

The associated effects are:

```text
Damage
Defense
Poise
Apply Status
Mana Recovery
Life Steal
Movement
Critical
Resistance
Phantom Hit
```

Sigils modify the relationships between these triggers and effects.

## General Sigils

| Sigil | Proposed behavior |
| --- | --- |
| Echo Sigil | Reduces the linked effect's potency, but allows it to activate again after X actions. |
| Conversion Sigil | Converts part of a weapon's damage type into another type already supported by its material/Core. |
| Retaliation Sigil | An effect originally associated with On Hit may be associated with On Damage Taken, with a penalty. |
| Execution Sigil | Strengthens the effect when the enemy is below a specified HP percentage. |
| Reserve Sigil | Delays immediate activation, accumulates activations up to a limit, and releases them all at the next valid activation. |
| Reciprocity Sigil | Increases the effect, but a fraction also negatively affects the user. |

## Weapon-Category Sigils

The intent is more than one isolated bonus per weapon: every template should have at least one exclusive property that other templates cannot normally obtain. These properties make a Greatsword and a Giant Axe meaningfully different weapons, beyond their numerical values.

| Weapon template | Exclusive Sigil | Proposed behavior |
| --- | --- | --- |
| Single Hand Light | Opportunist | After Dodge/Parry, the next On Hit may execute certain Sigils immediately without waiting for their normal trigger. |
| Dual Hand Light | Twin Echo | On Hit effects alternate between the main and secondary hands; the second activation gains properties different from the first. |
| Lightsword | Finesse | During a Critical, part of the secondary Scaling may be converted into primary Scaling. |
| Broadsword | Versatility | Allows switching between two Sigil configurations outside the action itself without changing weapons. |
| Greatsword | Momentum | Consecutive hits without Dodge increase Poise/penetration; Dodge resets Momentum. |
| Axe | Cleave | Part of excess overkill damage may hit another target instead of being wasted. |
| Giant Axe | Sundering | Damage to Tenacity accumulates a mark; breaking Tenacity consumes the marks for a brutal effect. |
| Spear | Reach | On Hit may be conditional on position/distance. Hitting at the ideal distance improves the associated effect. |
| Bow | Draw | May voluntarily lose Priority to improve the next shot's properties. |
| Crossbow | Loaded | Sigils may be associated with loaded ammunition; reloading prepares effects for the next shot instead of activating them immediately. |
| Shield | Reversal | Perfect Block may convert part of the blocked damage into a resource for the next attack/Art of War. |
| Greatshield | Bastion | Accumulates Guard while taking no offensive action; On Block effects consume Guard to increase their strength. |
| Scythe | Reaping | On Kill captures part of the dead target's status/lost HP as a resource for the next victim. |
| Colossal Weapon | Aftershock | The impact creates a delayed second instance based on the damage/Tenacity inflicted by the first hit. |

## Armor Sigils

The proposed distinction is by weight class rather than individual armor piece.

| Armor class | Exclusive Sigil | Proposed behavior |
| --- | --- | --- |
| Very Light | Untouched | A turn without taking damage accumulates a defensive/offensive bonus; being hit breaks it. |
| Light | Flowstep | A successful Dodge may feed the next equipment trigger. |
| Normal | Adaptation | Repeatedly receiving the same Damage Type temporarily increases resistance against it, while losing adaptation against the other types. |
| Heavy | Retention | Stores part of the Poise/Tenacity lost; On Hit may consume the reserve. |
| Very Heavy | Immovable | Prevented Push/Pull is converted into temporary Poise or Tenacity. |
| Robe | Resonance | The first Glyph of each spell gains an interaction with the Robe's Sigils. Robes are the armor class with the strongest connection to magic Nodes. |

Robe should have a richer identity than simply "armor with low defense and INT +10."

## Open Design Details

The source does not define the numeric strengths, resource limits, mark behavior, exact switching timing, or ownership rules of the proposed Sigils. Trigger and effect names above are retained design vocabulary. Their mapping to the canonical event system, damage stages, attributes, and status definitions must be specified before implementation; this catalog does not silently establish that mapping.

