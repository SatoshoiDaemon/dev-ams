# Numeric Design Framework

This document defines the first structural plan for deterministic gameplay numbers. It does not replace the canonical values already defined in the system documents. New values below are marked as **candidate defaults** and must remain configurable until playtesting establishes stable balance.

## Goals

The numeric layer must be:

- deterministic across Windows and Linux;
- explainable through combat logs and `explain last`;
- serializable in human-readable data;
- usable by base content and mods through the same contracts;
- resistant to accidental overflow and infinite trigger loops;
- explicit about rounding, stacking, and evaluation order. Limits are documented only when required by a mechanic, data safety, or an explicit design rule.

Balance is not an architectural requirement. Do not introduce hard caps, soft caps, diminishing returns, progression ceilings, or arbitrary numeric limits solely to contain balance. A limit must be required by the mechanic itself, by data safety, or by an explicit design decision.

## Numeric Domains

| Domain | Representation plan | Global behavior |
| --- | --- | --- |
| HP, Shield, Tenacity, Mana, Energy, damage, healing | Non-negative integer points | Clamp at the owning resource bounds after each resource stage. |
| Attribute points and Power | Non-negative integers | Attribute-to-Power conversion uses configured integer ratios. |
| Potency and Counter | Non-negative integers | Default range is `0–99`; individual effects may override or fix either field. |
| Weight and Priority | Signed integers | Weight is non-negative; final Priority may be negative. |
| Percent modifiers | Integer basis points | `100` basis points = `1%`; `10,000` = `100%`. |
| Ratios and Scaling | Integer numerator and denominator | Evaluate with checked integer multiplication followed by floor division. |
| Turn, action, tick, and trigger counts | Non-negative integers | Increment only at explicitly named events. |

Floating-point values should not cross the gameplay-data boundary. A TOML or JSON decimal should be parsed into a validated rational or basis-point value before simulation.

## Floor Rounding

The global rule remains:

```text
result = floor(numerator / denominator)
```

Perform the floor operation at the boundary of each named pipeline stage, because logs and mods need stable intermediate values. Do not repeatedly round inside one stage unless the formula explicitly defines multiple operations.

For reductions, calculate a non-negative magnitude first and then subtract it. This avoids language-dependent behavior for negative integer division.

```text
reduction = floor(value × reduction_bps / 10_000)
result = value - reduction
```

## Checked Arithmetic and Limits

The implementation should use a gameplay numeric wrapper or checked helper functions rather than scattered primitive arithmetic. Each helper must report the operation, operands, source, and configured limit on failure.

Every configurable numeric field needs:

- an accepted minimum and maximum;
- a default;
- a unit;
- a rounding point;
- an overflow policy;
- a clear error when invalid data is loaded.

Saturating arithmetic should be used only when saturation is itself the documented rule. Silent overflow and silent saturation are invalid.

## Modifier Model

Each modifier should declare its operation instead of being stored as an ambiguous number:

```text
FlatAdd
FlatSubtract
AdditivePercent
MultiplicativeRatio
Override
Minimum
Maximum
```

Within one damage-pipeline stage, use this candidate deterministic order:

```text
stage input
↓
flat additions and subtractions
↓
sum of additive-percent modifiers
↓
multiplicative ratios in stable source-ID order
↓
explicit minimum/maximum constraints
↓
floor and stage output
```

This is a structural candidate. Individual mechanics may define another operation order, but the exception must be represented in data or code as an explicit engine primitive and must appear in logs.

## Authoritative Damage Composition and Stages

Numeric planning must preserve the established order:

```text
Base Damage + Scaling Contributions
Equipment and Attributes as declared sources
Offensive Modifiers
Defensive Modifiers
Damage Type
Block
Shield
Tenacity
HP
On Damage Triggers
On HP Change Triggers
Death Check
```

Each stage produces an immutable explanation entry containing at least its input, operations, sources, rounding, output, and emitted events. Glyphs and Lua callbacks must attach to a named stage or event; they cannot insert an undocumented calculation between stages.

## Existing Canonical Baselines

The following values are already defined and are not new proposals:

| Rule | Value |
| --- | --- |
| HP per Vigor | `10` |
| Tenacity per Resistance | `10` |
| MP per Mana | `10` |
| Power per offensive attribute point | `7` |
| Scaling D / C / B / A / S / SS / SSS | `0.25 / 0.50 / 0.75 / 1.00 / 1.50 / 2.00 / 2.50` |
| Normal Tenacity split | `50% HP / 50% Tenacity` |
| True Damage | Bypasses Tenacity |
| Default status limits | Potency `0–99`, Counter `0–99` |
| Competing status order | Potency first, Counter second |
| Attribute enhancement sources | Maximum `4` per attribute |

The canonical attribute ID is `intelligence`; legacy `spirit` save data migrates one-for-one to `intelligence`. The migration does not change the `7 Power` conversion.

## Status Numeric Contract

Every status definition must declare:

```text
id
category
uses_potency
uses_counter
potency_min
potency_max
counter_min
counter_max
fixed_potency
application_rule
tick_events
decay_operations
expiry_condition
modifier_operations
damage_type, when damage is produced
source tracking requirements
```

The engine must not assume universal end-of-turn decay. Each effect lists the event that causes a tick and the event that changes Potency or Counter.

When a Glyph changes a status, it produces a proposed status transformation. The status engine validates the result, applies competition, records the winning source, and emits the appropriate events.

## Configurable Curves

Soft caps, diminishing returns, experience curves, Priority contribution, equipment growth, and economy progression should use named, versioned curve definitions. The first supported primitives should be:

- piecewise linear;
- rational polynomial with bounded input;
- lookup table with deterministic interpolation rules;
- stepped thresholds.

Avoid a general expression language in the first version. Complex custom behavior belongs behind the versioned Lua API after the safe event and numeric contracts exist.

## Balance Measurements

Each offensive or defensive option should be measurable in common units without claiming that all effects are equivalent:

| Measurement | Purpose |
| --- | --- |
| Immediate damage equivalent | Compare direct output at the same progression point. |
| Expected lifetime output | Evaluate Counter-based effects under stated assumptions. |
| Resource swing | Sum resource gained and resource denied, with the resource type preserved. |
| Action control | Count actions or action categories denied, delayed, or redirected. |
| Positional control | Count forced movement, prevented movement, or affected spaces. |
| Propagation bound | Maximum targets, repeats, chains, or created instances. |
| Output per Weight | Compare spell structures at the same capacity cost. |

Action control and direct HP bypass must remain separate high-impact measurements rather than being hidden inside a single damage-equivalent formula.

## Required Test Vectors

Every approved formula needs table-driven tests that include:

- zero, minimum, normal, cap, and above-cap input;
- fractional results immediately below and above an integer boundary;
- multiple modifiers from stable source IDs in different registration orders;
- Shield depletion, Tenacity depletion, and HP overflow damage;
- True Damage with and without Shield;
- status replacement on lower, equal, and higher Potency/Counter;
- Glyph transforms at caps and with insufficient Counter/Potency;
- event insertion at the maximum trigger depth;
- serialization and reload of all intermediate state required by the formula.

## Approval Path

For each system, planning should progress through four explicit states:

```text
Missing
→ Structural candidate
→ Candidate defaults with test vectors
→ Canonical and versioned
```

A formula becomes canonical only when its owning system document, configuration schema, examples, and regression tests agree. Playtesting may change defaults without changing the engine primitive or serialized contract.
