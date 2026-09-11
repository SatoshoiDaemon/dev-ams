# Glyph Numeric Planning

This document is the first structural candidate for Glyph numbers. It converts the existing behavior catalog into deterministic, configurable contracts without declaring final balance. The canonical Glyph descriptions remain in [Magic Nodes and Glyphs](../systems/glyphs.md), and status behavior remains in [Status Effects](../systems/effects.md).

## Separate Weight from Status Budget

Two values serve different purposes:

- **Status Budget** measures the configured Potency/Counter value carried by one status application.
- **Spell Weight** measures the capacity and action burden of the complete spell tree, including Nodes, parameters, Glyphs, and Sacrifices.

A Glyph may preserve Status Budget while still increasing Spell Weight. These values must not share one ambiguous `cost` field.

## Candidate Spell Weight Equation

```text
Spell Weight = max(
    Minimum Spell Weight,
    Σ Node Base Weight
  + Σ Parameter Weight
  + Σ Glyph Weight
  - Σ Sacrifice Credit
)
```

Every term is an integer. Each registered source contributes an explanation entry. Sacrifice Credit cannot reduce a branch below its configured minimum.

For a non-negative parameter, the candidate bucket cost is:

```text
bucket(value, step) =
    0, when value = 0
    1 + floor((value - 1) / step), otherwise
```

A candidate status Node can use:

```text
Status Node Weight =
    Effect Base Weight
  + bucket(Potency, Potency Step)
  + bucket(Counter, Counter Step)
```

The steps and base Weight belong to each status definition. This prevents Stun, Shield, Burn, and Poison from being priced as if their values had identical consequences.

### Calibration Against the Existing Example

The existing interface shows `Burn 25/12` with Weight `18`. The following candidate values reproduce it:

```text
Burn Base Weight = 5
Potency Step = 3
Counter Step = 3

5 + bucket(25, 3) + bucket(12, 3)
= 5 + 9 + 4
= 18
```

The later example shows Condensed, Accelerated, and Ignition raising the same spell to Weight `31`. A candidate calibration is:

```text
Base Burn 25/12 = 18
Condensed = 4
Accelerated = 4
Ignition = 5

18 + 4 + 4 + 5 = 31
```

These values are a useful first test vector because they preserve both existing examples. They remain candidate defaults until compared with other statuses and direct-damage spells.

## Candidate Status Budget Equation

Status Budget needs effect-specific exchange rates:

```text
Status Budget =
    Potency × Potency Rate
  + Counter × Counter Rate
```

Rates are non-negative configured integers. A redistribution Glyph declares an efficiency ratio:

```text
Output Budget <= floor(Input Budget × Efficiency / 10_000)
```

`10,000` efficiency basis points preserve the input budget. Higher efficiency must have an explicit Weight, resource, condition, or timing cost. Lower efficiency intentionally loses value.

The existing examples demonstrate why rates cannot be universal:

```text
Burn 20/20 → Condensed → Burn 35/8
```

Candidate Burn rates of `4 Potency / 5 Counter` preserve budget:

```text
20 × 4 + 20 × 5 = 180
35 × 4 + 8 × 5 = 180
```

The Poison example preserves budget with equal rates:

```text
Poison 30/10 → Lingering → Poison 15/25

30 × 1 + 10 × 1 = 40
15 × 1 + 25 × 1 = 40
```

These are candidate calibration rates, not final claims that one Burn point is worth four or five units across the entire game.

## Candidate Glyph Weight Classes

| Class | Candidate base Weight | Typical behavior |
| --- | ---: | --- |
| Conditional permission | `2` | Adds a meaningful requirement before the effect can occur. |
| Geometry or trajectory change | `3` | Repositions, redirects, or changes targeting without duplicating full output. |
| Timing change | `4` | Delays, accelerates, or moves an existing effect to another event. |
| Budget redistribution | `4` | Exchanges Potency, Counter, resource, or another bounded value. |
| Rule replacement | `5` | Replaces a normal tick, decay, or interaction rule. |
| Propagation | `6` | Copies or spreads bounded partial output to another target. |
| Defense bypass or execution | `7` | Bypasses a defensive layer or converts a death/break into output. |
| Additional execution | `8` | Repeats an effect or creates another scheduled execution. |

The final Weight of a Glyph is its class Weight plus parameter surcharges. Conditions may grant limited credit, but a condition cannot make a positive-output Glyph free.

## General Status Glyph Candidates

| Glyph | Candidate class/Weight | Required parameters |
| --- | --- | --- |
| Condensed | Budget redistribution, `4` | Target Potency/Counter ratio, efficiency, minimum Counter |
| Lingering | Budget redistribution, `4` | Target ratio, efficiency, minimum Potency |
| Stable | Rule replacement, `5` | Protected ticks, affected decay operations, maximum prevention |
| Accelerated | Timing change, `4` | Immediate executions, Counter consumed, event timing |
| Delayed | Conditional/timing, `3` | Delay actions/turns, Potency gain, cancellation behavior |
| Contagious | Propagation, `6` | Transfer ratio, range, target count, selection order |
| Embedded | Rule replacement, `6` | Removal event, output type, ratio, maximum output |
| Threshold | Conditional permission, `2` | Condition type/value, comparison, efficiency reward |

No Glyph may be applied to itself repeatedly unless its definition explicitly permits a bounded number of ranks. Condensed and Lingering must not form a loop that creates Status Budget.

## Elemental Numeric Cards

Each existing elemental catalog is preserved. The next pass must fill the following fields before assigning final Weights:

| Element | Numeric decisions required |
| --- | --- |
| Fire | Burn Counter consumed by Ignition; Kindling storage ratio/cap; Flashover transfer and targets; Cremation conversion/cap; White Flame damage penalty and Shield coefficient |
| Water | Chain return efficiency; Dilution exchange; overheal conversion/cap; Undertow Priority penalty; Equalize conservation and target order |
| Earth | Foundation stationary bonus; Seismic distance conversion; Faultline transfer; Bedrock Shield penalty/Resistance Scaling; Erosion step/cap/reset |
| Air | Tailwind Priority coefficient; Slipstream gain; Ricochet efficiency/targets; Vacuum movable-object limit; Updraft damage exchange and interrupt threshold |
| Light | Purification heal exchange; Revelation lockout Counter; Judgment conversion; Sanctuary share ratio; Absolution resource restoration/cap |
| Darkness | Devour conversion; Corruption conversion; Shadow Debt timing/cap; Usurpation duration/selection; Eclipse isolation range and efficiency |
| Poison | Debuff contribution to Toxic Catalyst; Necrosis HP bypass and Potency loss; Anticoagulant trigger amount; Reservoir preserved Counter; Compound decay reduction |
| Ice | Permafrost protected event; Brittle Counter cost and Tenacity gain; Cold Snap conversion; Cryostasis duration/cost; Shatter conversion/cap |
| Sound | Echo delay and efficiency; Resonance step/cap/reset; Dissonance interrupt value; Harmonic target curve; Feedback return ratio/cap |
| Metal | Magnetism equipment coefficient; Sharpen damage/penetration exchange; Temper armor curve; Shrapnel count/efficiency; Forge Bond Scaling conversion |
| Plant | Growth step/cap; Root movement conversion; Germination child size/generations; Photosynthesis Mana/cap; Propagation output and targets |
| Lava | Molten Ground tick/area duration; Obsidian output; Melt Armor step/cap; Viscosity movement exchange; Eruption remaining-Counter conversion |
| Lightning | Conductor network limits; Overload Counter conversion; Arc Jump target/efficiency; Grounding single-target formula; Impulse Priority coefficient |

Every row should become five data-backed Glyph numeric cards. The behavior catalog therefore requires 65 elemental cards plus the eight general status cards.

## Glyph Numeric Card

Each Glyph should be planned and stored with fields equivalent to:

```text
id
name
allowed_node_tags
required_element
weight_class
base_weight
parameter_surcharges
trigger_event
trigger_limit_per_event
target_limit
input_snapshot_rule
consumed_values
produced_values
efficiency_ratio
minimum_output
maximum_output
cooldown_actions_or_rounds
stacking_rule
reset_rule
status_competition_timing
damage_pipeline_stage
log_fields
```

Unused fields should be absent rather than filled with magic sentinel values.

## Status Transformation Order

Candidate order for an incoming status:

```text
Build base status from the spell
↓
Validate element and Node eligibility
↓
Apply bounded Glyph budget transformations
↓
Clamp to effect-specific limits
↓
Compare with the target's competing instance
↓
Apply or discard the incoming instance
↓
Emit Status Applied or competition result
↓
Schedule later ticks and triggers
```

A Glyph that consumes an already active status must use an explicit operation such as `ConsumeCounter`, `ConsumePotency`, `TransferCounter`, `CopyPotency`, or `FreezeDecay`. It must not mutate status fields through an unlogged generic callback.

## Deterministic Trigger Safety

Candidate engine safety defaults:

```text
Maximum trigger depth per action: 16
Maximum emitted combat events per action: 256
Maximum activation of one Glyph instance per source event: 1
```

These values are safety candidates, not balance rewards. They must be configurable within validated bounds and should fail the current action with a useful diagnostic rather than crash or hang the game.

Target selection, Chain order, propagation, and Ricochet must use a stable ordering rule or the explicit seeded RNG service. Never depend on registry insertion order, hash-map iteration order, or terminal presentation order.

## Required Glyph Test Vectors

- Reproduce Weight `18` for the candidate `Burn 25/12` calibration.
- Reproduce Weight `31` after candidate Condensed, Accelerated, and Ignition costs.
- Preserve candidate Burn budget for `20/20 → 35/8`.
- Preserve candidate Poison budget for `30/10 → 15/25`.
- Reject a transform above the status cap or below its required minimum.
- Prove Condensed followed by Lingering cannot increase budget through repetition.
- Apply a lower-Potency status with greater Counter and confirm the existing status wins.
- Apply equal Potency with greater Counter and confirm the incoming status wins.
- Confirm an elemental Glyph is rejected without the required elemental branch.
- Confirm propagation and repeat behavior stop at configured target, activation, event, and depth limits.
- Confirm every Weight and status transformation appears in `explain last` with source IDs.

## Next Glyph Planning Pass

1. Approve or replace the candidate Weight and Status Budget equations.
2. Choose effect-specific base Weights and Potency/Counter rates for all 25 statuses.
3. Approve the eight general Glyph cards and their parameter bounds.
4. Complete one element at a time, starting with Fire, Water, and Earth as contrasting damage, redistribution, and defense/control cases.
5. Compare each completed element against direct-damage and no-Glyph reference spells at the same Weight.
6. Move approved defaults into versioned data only after worked examples and regression vectors agree.
