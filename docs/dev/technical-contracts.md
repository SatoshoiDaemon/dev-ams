# Technical Contracts

This page defines the minimum technical contracts shared by the systems in `docs/systems`. It does not add gameplay rules. A system page marked **Canonical** is authoritative over proposal and historical text.

## Authority

When documents disagree, use this order:

```text
Canonical > Proposal > Historical
```

- **Canonical**: implementation, tests, saves, logs, and mods must follow it.
- **Proposal**: design material that is not an implementation instruction until promoted.
- **Historical**: preserved provenance only; it never overrides a canonical rule.
- **Open contract**: a technical decision still required before implementation.

Every system page should identify its status near the top. Historical sections should say explicitly that they are retained for provenance.

## Stable IDs

Persistent content uses the form:

```text
namespace:local_id
```

IDs are case-sensitive, must contain exactly one namespace separator, and must not depend on registry order or array position. The `base` namespace belongs to the game. A mod owns IDs in its declared namespace. Duplicate IDs are a load error unless an explicit replacement mechanism is added later.

References in saves, content files, logs, and Lua use stable IDs. Renaming a persistent ID requires a migration alias.

## Content data and validation

Content may use JSON or TOML according to the owning system. Each content type defines a small schema containing its required fields, field types, valid ID references, and bounds. A recipe, for example, only needs inputs, output, and a stable ID; it does not implicitly gain quality, budgets, or failure rules.

Unknown or invalid external data must produce a player-readable diagnostic containing:

```text
mod, file, object ID, field, received value, and reason
```

Invalid content is rejected without a panic. One invalid mod should not make unrelated base content uninspectable, but the load result must clearly identify that the mod was not loaded completely.

## Mod manifest and load order

Every mod has a manifest with at least:

```toml
id = "author:example"
name = "Example Mod"
version = "1.0.0"
api_version = 1
```

Dependencies may be declared by stable ID. Loading is deterministic:

```text
engine primitives
→ base content
→ dependencies in dependency order
→ independent mods by stable ID
→ each mod's files by stable path order
```

Missing dependencies, dependency cycles, invalid manifests, and duplicate IDs are load errors. A mod may replace existing content only through an explicit replacement declaration; ordinary registration cannot silently overwrite another ID.

## Saves and migrations

Every save records at least:

```json
{
  "save_format_version": 1,
  "ruleset_version": "..."
}
```

`save_format_version` describes serialized structure. `ruleset_version` identifies the gameplay defaults used to interpret it. Modded saves also record the required mod IDs and versions when that information is needed to load their content.

Loading runs migrations in version order. A migration may rename fields or IDs, add defaults, and emit a warning. A save is rejected only when it cannot be made safe to load; the diagnostic identifies the save, version, field, and reason. Save data remains human-readable and editable.

## Lua boundary

Lua accesses the game only through the versioned mod API:

```text
mod_api_version = 1
```

The API exposes stable IDs, documented value objects, registration functions, and event callbacks. It does not expose Rust internals, arbitrary filesystem access, process execution, shell commands, native libraries, or implicit operating-system access.

A Lua error identifies the mod, script, line, callback/event, and reason. The failing callback is disabled or the affected operation is rejected according to the owning system; the whole game must not crash because of an ordinary mod error. Callback recursion remains subject to the combat/event safety limits.

## Event contract

Events have a stable name and a documented payload. The core combat event names are:

```text
OnAttackReceived
OnEvade
OnParry
OnBlock
OnDamageReceived
OnShieldDamage
OnTenacityDamage
OnHPDamage
OnHPBelow
OnKill
```

`OnAttackReceived` is the public name for an attack that effectively connected; it is not emitted when Evasion or Parry prevents the attack. `OnDamageReceived` requires damage to actually reduce at least one receiving damage layer; damage completely absorbed by Shield does not emit it. Layer events require that their resource actually decreased. `OnHPBelow` has no universal once/while-below behavior; the effect or listener definition specifies how it consumes the threshold event. Events produced by a Sigil are new events and may activate eligible sources subject to the global recursion and event-count limits.

Event processing follows the owning canonical system's order. Listeners are processed deterministically by registered stable ID. Events created during an event are queued after the current event and remain subject to trigger-depth and event-count limits.

## Targeting

Targeting is selection-based, not spatial. Combat does not simulate distance, positioning, line of sight, line of effect, obstacles, or attack ranges unless a specific action explicitly implements such a requirement.

The canonical target specification is:

```text
TargetSpec:
    mode: Self | Single | Multiple(N) | All
    relation: Self | Ally | Enemy | Any
    state: Active | Dead | Any
    filters: ActionRequirement[]
```

`Self` selects only the acting entity. `Single` requires exactly one valid selected target. `Multiple(N)` accepts up to `N` selected targets. `All` determines the set at resolution rather than storing a concrete list at declaration time. Content may name filtered forms such as `AllEnemies`, `AllAllies`, or `Everyone`; these are relation-based selections, not geometric areas.

By default, only active targets are valid. Dead or removed targets are invalid. Resurrection actions explicitly opt into `state: Dead` rather than changing the generic targeting rules. Relations are evaluated from the actor and target at validation time.

Single and Multiple targets are selected during action declaration. All-target selections are evaluated at resolution. The initial TargetSpec validation happens before AP and resource payment. Every action then revalidates its targets at resolution:

```text
target exists
AND target is active or matches the declared state
AND action permits the target relation
AND action-specific requirements are satisfied
```

Invalid selected targets are removed before resolution. A Single action fails when its target is invalid. A Multiple action resolves against the remaining valid targets and fails only when no valid target remains. An All action resolves against the valid set found at resolution.

Target requirements are declarative and action-specific, such as `target_has_status`, `target_is_boss`, or `target_hp_below`. They must not introduce a hidden global spatial system.

## Explain Last

`explain last` exposes the latest completed action packet, whether the action resolved or was rejected. It replaces the previous packet; it is not an unbounded combat history and is not required to be persisted in the save. For delayed effects such as Aftershock, `after 1 action` counts the next action resolved by any combatant.

The packet includes, when applicable:

```text
action declaration and validation
actor, source, target IDs, and declared round
AP/resource payment
Cast and eligible round
Priority contributions and queue position
Accuracy, Evasion, Parry, and Block inputs/results
Scaling sources and floored contributions
offensive and defensive modifiers
Defense and penetration
Shield, Tenacity, and HP before/after
events emitted and queued
trigger order, depth, and rejection reason
ruleset version and RNG seed/stream when randomness was used
```

Absorbed, skipped, rejected, or inapplicable stages are represented explicitly rather than silently omitted. A future aggregate `explain turn` command may summarize multiple actions; it is distinct from `explain last`.

## Remaining open technical contracts

The following are not resolved by this page:

- fleeing and encounter-specific escape conditions;
- the complete module boundary between engine, data, persistence, terminal presentation, and modding;
- whether a particular system needs to preserve mod-owned unknown save fields during migration;
- final Lua API operation lists for each gameplay system;
- implementation tests and balance validation for the canonical numeric values.
