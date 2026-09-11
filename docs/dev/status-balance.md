# Status Numeric Planning

This document audits the 25 canonical base Status Effects and defines the next numeric work required for each one. It does not replace [Status Effects](../systems/effects.md). Existing formulas below are canonical; proposed structures are marked as candidates.

## Shared Status Lifecycle Candidate

Each status definition must explicitly select events for application, execution, decay, and expiry:

```text
Build incoming instance
↓
Apply source and Glyph transformations
↓
Validate effect-specific limits
↓
Resolve Potency/Counter competition
↓
Emit application result
↓
Execute only on declared events
↓
Apply declared decay operations
↓
Expire when the effect's declared condition is true
```

Candidate default expiry for a status that uses Counter is `Counter == 0`. Potency reaching zero expires the status only when its definition declares that condition. Instant operations such as Heal need an explicit `instant` lifecycle instead of pretending to have a duration.

## Debuff Resistance Curve Candidate

Resistance/Tenacity reduces debuff duration, but the curve is not yet defined. A deterministic rational candidate is:

```text
Duration Multiplier =
    Curve Constant / (Curve Constant + Relevant Tenacity)

Final Counter =
    max(Minimum Counter, floor(Base Counter × Duration Multiplier))
```

With a candidate Curve Constant of `1,000` and Minimum Counter of `1`:

```text
Tenacity 0:    Counter 20 → 20
Tenacity 500:  Counter 20 → 13
Tenacity 1000: Counter 20 → 10
Tenacity 2000: Counter 20 → 6
```

This is a curve candidate for comparison, not an approved default. Planning must decide whether the input is current Tenacity, maximum Tenacity, Resistance-derived Tenacity, or a dedicated resistance value. Crowd Control may require its own curve or minimum.

## Damage over Time

| Status | Existing numeric rule | Missing numeric decision |
| --- | --- | --- |
| Burn | Magical Damage = Potency; loses `1` Potency and `1` Counter per turn | Exact tick/decay event order and whether a zero-Potency instance expires |
| Poison | Magical Damage = Potency × continuous poisoned turns; Counter halves with floor | Tick event, first-turn timing, replacement history, and whether application resets continuous turns |
| Sinking | Loses Potency Energy/Mana; Counter loses `1` per turn | Resource-selection rule, tick event, minimum resource result, and zero-resource behavior |
| Electrified | Magical Damage = Potency × Electrified targets; Counter loses `1` per turn | Battlefield scope, target-count snapshot event, summon inclusion, network cap, and tick order |
| Bleed | Damage = Potency when the target attacks or successfully evades; Potency and Counter halve with floor | Damage type, exact attack/evade event, decay event order, and behavior at Potency `0` |

## Crowd Control

| Status | Existing numeric rule | Missing numeric decision |
| --- | --- | --- |
| Stun | Potency fixed at `1`; Counter maximum `5`; blocks attacks and evasion | Counter decay event, skipped-action semantics, resistance curve, and reapplication timing |
| Binding | Potency fixed at `1`; Counter maximum `5`; blocks movement and physical actions | Counter decay, classification of mixed actions, resistance curve, and movement-conversion bounds |
| Silent | Prevents Magic | Potency use/fixed value, Counter cap and decay, resistance curve, and definition of Magic actions |
| Fear | Cannot attack source; Evasion reduced `1%` per Potency; Counter loses `1` per turn | Potency cap, source-loss behavior, decay event, and action selection when no legal attack exists |
| Taunt | Can attack only source; damage dealt reduced `1%` per Potency; Counter loses `1` per turn | Potency cap, source-loss behavior, decay event, and rules for non-attack actions |
| Charm | Cannot attack source; source gains `1%` damage per Potency against target | Counter cap/decay, Potency cap, source-loss behavior, and damage-pipeline stage |

## Buffs

| Status | Existing numeric rule | Missing numeric decision |
| --- | --- | --- |
| Protect | Damage Reduction `+1%` per Potency; Counter loses `1` per turn | Reduction cap, stacking stage, decay event, and interaction with True Damage |
| Haste | Evasion `+1%` and proportional Priority per Potency; Counter loses `1` per turn | Priority coefficient, Evasion cap, decay event, and tie-break effects |
| Rage | Physical Damage `+1%` per Potency; Counter loses `1` per turn | Modifier stage, cap, decay event, and affected physical sources |
| Overdrive | Mana Regeneration `+1%` per Potency; Counter loses `1` per turn | Base regeneration unit, rounding, cap, decay event, and zero-regeneration behavior |
| Sage | Magical Damage `+1%` per Potency; Counter loses `1` per turn | Modifier stage, cap, decay event, and affected magical sources |
| Heal | Heals `1%` maximum HP per Potency; no Counter | Instant lifecycle, rounding, overheal, competition relevance, and maximum Potency |
| Shield | Grants `10` Shield HP per Potency; unlimited Potency | Counter use/decay, Shield ordering, replacement/stacking, source separation, and numeric storage limit |
| Blessing | Healing and Shield HP `+1%` per Potency; Counter loses `1` per turn | Modifier stage, cap, decay event, and whether existing Shield is recalculated |

## Debuffs

| Status | Existing numeric rule | Missing numeric decision |
| --- | --- | --- |
| Fragile | HP Damage received `+1%` per Potency; Counter decreases by Potency per turn | Decay event, HP-only stage placement, cap, and minimum useful lifetime |
| Tremor | General Damage received `+1%` per Potency; Counter loses `1` per turn | Stage placement, cap, decay event, and interaction with Fragile |
| Rupture | Ignores `1%` Tenacity per Potency; Potency loses `1` per turn | Bypass cap, Counter behavior, decay event, and split calculation order |
| Exhaust | Attack Speed and Evasion `-1%` per Potency; Counter loses `1` per turn | Attack Speed definition, Evasion floor, decay event, and Priority relationship |
| Attack Down | Damage dealt `-1%` per Potency; Counter loses `1` per turn | Damage floor, modifier stage, cap, decay event, and affected sources |
| Curse | Modifies other Debuff efficiency by Counter; Counter loses `1` per turn | Resolve the contradictory increase/decrease wording, define target values, order, cap, and recursion behavior |

## Status Card Additions

In addition to the fields already required by the status data model, numeric planning should record:

```text
application_event
first_tick_event
repeat_tick_event
decay_event
expiry_conditions
resistance_curve_id
minimum_effective_counter
modifier_stage
modifier_cap
damage_type
snapshot_or_live_inputs
replacement_state_policy
source_loss_policy
```

## Required Cross-Status Tests

- Protect, Tremor, Fragile, Attack Down, Rage, and Sage in the same damage calculation with every modifier shown by stage.
- Stun, Binding, Silent, Fear, Taunt, and Charm when an actor has no legal declared action.
- Burn and Poison applied at the same time with deterministic tick order.
- Poison replacement before and after its tick once a replacement policy is approved.
- Electrified with actors dying, spawning, or losing the status during the same event sequence.
- Bleed triggered by an attack that misses, is blocked, is parried, or is evaded.
- Heal with fractional maximum-HP output and overheal conversion.
- Shield creation, replacement, depletion, and expiry before Tenacity/HP processing.
- Rupture values at `0%`, the approved cap, and above-cap invalid input.
- Curse interacting with another Curse and with a debuff modified by a Glyph.

## Completion Order

1. Resolve Curse and the Spirit/Intelligence dependency used by magical effects.
2. Assign lifecycle events and expiry conditions to all 25 effects.
3. Approve damage types, modifier stages, caps, and resistance curve inputs.
4. Approve replacement-state behavior for Poison and source-loss behavior for source-linked control.
5. Define status-specific Potency and Counter rates for the Glyph Status Budget model.
6. Add worked examples and regression vectors to each canonical effect section.
