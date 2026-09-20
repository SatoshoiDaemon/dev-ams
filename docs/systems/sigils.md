# Equipment Sigils

> **Specification status:** The canonical section at the end supplies the numeric event contract for Sigils. The catalog above preserves original names and intent.

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

## Canonical Quantified Sigil Contract

Every Sigil stores `owner_id`, `item_id`, `trigger_event`, `effect`, `potency`, `counter`, `cooldown`, `activation_limit`, and `status_packet` fields. A Sigil activates at most once per matching event, has a default cooldown of `0`, and cannot recursively activate itself. Equipment triggers use the event names `OnAttackReceived`, `OnDamageReceived`, `OnHPDamage`, `OnTenacityDamage`, `OnShieldDamage`, and `OnHPBelow`, plus the catalog events On Hit, On Critical, On Block, On Parry, On Dodge, On Kill, and While Equipped. Status effects are applied only through the shared StatusPacket contract; a Sigil never creates an unregistered anonymous effect.

General Sigils have these fixed values: Echo `Weight 3`, effect `50%`, delay `2 actions`, one repeat; Conversion `Weight 3`, converts `25%` of Physical/Magical output to the selected supported type; Retaliation `Weight 3`, activates at `50%` effect on OnDamageReceived; Execution `Weight 3`, activates at `+25%` when target HP is `<=25%`; Reserve `Weight 3`, stores `3` activations and releases on the next matching event at full summed effect; Reciprocity `Weight 3`, grants `+25%` effect and deals `10%` of that effect to owner as HP Damage.

Weapon-category Sigils use the following canonical values: Opportunist `+1 immediate activation`; Twin Echo second hand `-25%` effect; Finesse converts `20%` secondary Scaling; Versatility stores 2 configurations and switching costs `0 Action Points` outside an action; Momentum `+5 Poise and +5 Armor Penetration` per consecutive hit, cap `25`, reset on Dodge; Cleave transfers `20%` overkill to one target within 2 tiles, cap `30`; Sundering stores `10%` Tenacity damage, cap `50`, and on break deals the store as Physical Damage; Reach requires distance `2..4` and grants `+15%` effect; Draw loses `20 Priority` for `+25%` next-shot effect; Loaded stores one prepared activation; Reversal stores `20%` blocked damage, cap `30`, for the next attack; Bastion gains `5 Guard` per non-offensive action, cap `20`, consumed by On Block; Reaping stores `10%` target lost HP, cap `50`, for the next target; Aftershock repeats `30%` of first impact after `1 action`, where the action count advances on any action resolved by any combatant. These local caps are Sigil mechanics, not global balance caps.

Armor Sigils use: Untouched `+10%` next Sigil effect after one turn without HP damage, broken on damage; Flowstep `+15 Priority` after Dodge for one action; Adaptation `+5%` resistance to the last Damage Type per repeated hit, cap `20%`, and loses one stack when another type is received; Retention stores `25%` lost Tenacity, cap `30`, consumed by next On Hit; Immovable converts prevented movement at `5 Tenacity per tile`, cap `20`; Resonance reduces the first Glyph Weight in each spell by `1`, minimum Weight `1`.

Each Sigil result must log trigger, owner, item, input snapshot, consumed resource, produced value, StatusPacket IDs, cooldown, and rejection reason. Regression tests must cover every trigger, one-activation limits, reserves, cooldowns, status competition, and ownership after item swaps.
