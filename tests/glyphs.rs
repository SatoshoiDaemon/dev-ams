use ams::{
    effects::{status_budget, ActiveStatus, StatusRegistry},
    glyphs::{
        accelerated, amplify_status_repeated, condensed, evaluate_spell, lingering, Element,
        GlyphActivationGuard, GlyphError, GlyphRegistry, NodeKind, SpellBlueprint,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

fn registry() -> GlyphRegistry {
    GlyphRegistry::load(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/glyphs.json")).unwrap()
}

#[test]
fn base_catalog_contains_every_documented_card_with_positive_weight() {
    let registry = registry();
    assert_eq!(registry.len(), 76);
    assert!(registry.iter().all(|(id, card)| id.starts_with("base:")
        && card.weight > 0
        && !card.rule.is_empty()
        && !card.requirements.is_empty()));
    let mut groups: BTreeMap<Option<Element>, (usize, i64)> = BTreeMap::new();
    for (_, card) in registry.iter() {
        let group = groups.entry(card.element).or_default();
        group.0 += 1;
        group.1 += card.weight;
    }
    let expected = BTreeMap::from([
        (None, (11, 48)),
        (Some(Element::Air), (5, 21)),
        (Some(Element::Darkness), (5, 25)),
        (Some(Element::Earth), (5, 24)),
        (Some(Element::Fire), (5, 26)),
        (Some(Element::Ice), (5, 27)),
        (Some(Element::Lava), (5, 27)),
        (Some(Element::Light), (5, 23)),
        (Some(Element::Lightning), (5, 22)),
        (Some(Element::Metal), (5, 22)),
        (Some(Element::Plant), (5, 25)),
        (Some(Element::Poison), (5, 24)),
        (Some(Element::Sound), (5, 27)),
        (Some(Element::Water), (5, 20)),
    ]);
    assert_eq!(groups, expected);
}

#[test]
fn spell_weight_capacity_and_element_requirements_are_enforced() {
    let glyphs = registry();
    let blueprint = SpellBlueprint {
        id: "test:burn".into(),
        nodes: vec![
            NodeKind::Source,
            NodeKind::Type,
            NodeKind::Form,
            NodeKind::Effect,
        ],
        parameters: vec![25, 12],
        glyph_ids: vec!["base:condensed".into(), "base:ignition".into()],
        sacrifices: vec![],
        elements: BTreeSet::from([Element::Fire]),
        capabilities: BTreeSet::from(["apply_status".into(), "burn".into()]),
        target_count: 1,
        secondary_effect_count: 0,
    };
    let evaluation = evaluate_spell(&blueprint, &glyphs, 7, 1, 0).unwrap();
    assert_eq!(
        (
            evaluation.node_weight,
            evaluation.parameter_weight,
            evaluation.glyph_weight
        ),
        (5, 5, 9)
    );
    assert_eq!(
        (
            evaluation.final_weight,
            evaluation.capacity,
            evaluation.mana_cost
        ),
        (19, 39, 24)
    );
    let mut invalid = blueprint;
    invalid.elements.clear();
    assert!(matches!(
        evaluate_spell(&invalid, &glyphs, 7, 1, 0),
        Err(GlyphError::MissingElement { .. })
    ));
}

#[test]
fn condensed_and_lingering_match_budget_examples() {
    let mut burn = ActiveStatus::new("base:burn", "base:caster", 20, 20);
    let before = status_budget(&burn.effect_id, burn.potency, burn.counter).unwrap();
    condensed(&mut burn).unwrap();
    assert_eq!((burn.potency, burn.counter), (35, 8));
    assert_eq!(
        status_budget(&burn.effect_id, burn.potency, burn.counter).unwrap(),
        before
    );

    let mut poison = ActiveStatus::new("base:poison", "base:caster", 30, 10);
    lingering(&mut poison).unwrap();
    assert_eq!((poison.potency, poison.counter), (15, 25));
}

#[test]
fn repeated_status_amplification_is_additive_and_floored_once() {
    let mut two = ActiveStatus::new("base:poison", "base:caster", 20, 4);
    amplify_status_repeated(
        &mut two,
        &[("mod:first".into(), 2_500), ("mod:second".into(), 2_500)],
    )
    .unwrap();
    assert_eq!((two.potency, two.counter), (30, 6));

    let mut three = ActiveStatus::new("base:poison", "base:caster", 20, 4);
    amplify_status_repeated(
        &mut three,
        &[
            ("mod:first".into(), 2_500),
            ("mod:second".into(), 2_500),
            ("mod:third".into(), 2_500),
        ],
    )
    .unwrap();
    assert_eq!((three.potency, three.counter), (35, 7));
    let amplified_budget = status_budget(&three.effect_id, three.potency, three.counter).unwrap();
    condensed(&mut three).unwrap();
    assert!(
        status_budget(&three.effect_id, three.potency, three.counter).unwrap() <= amplified_budget
    );
}

#[test]
fn glyph_activation_is_once_per_source_event_and_accelerated_checks_counter() {
    let mut guard = GlyphActivationGuard::default();
    guard.activate(1, "base:ignition", "base:target").unwrap();
    assert!(matches!(
        guard.activate(1, "base:ignition", "base:target"),
        Err(GlyphError::RepeatedActivation { .. })
    ));
    let mut burn = ActiveStatus::new("base:burn", "base:caster", 10, 1);
    assert!(matches!(
        accelerated(&mut burn, &StatusRegistry::base()),
        Err(GlyphError::InsufficientCounter { .. })
    ));
}
