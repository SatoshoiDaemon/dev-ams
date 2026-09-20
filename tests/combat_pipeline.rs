use ams::{
    attributes::{AttributeRules, AttributeSet},
    combat::{
        apply_status_modifiers_to_damage, resolve_attack_action, resolve_damage,
        ActionTraceContext, AttackRequest, DamageRequest, DamageType, DeterministicRng,
        ExplanationStore,
    },
    content::StableId,
    effects::{ActiveStatus, StatusRegistry},
    entities::{Allegiance, Entity},
    events::CombatEvent,
    scaling::ScalingResult,
};

fn target() -> Entity {
    Entity::new(
        StableId::new("base:target").unwrap(),
        "Target",
        Allegiance::Enemy,
        AttributeSet::default(),
        &AttributeRules::default(),
    )
    .unwrap()
}

#[test]
fn complete_action_replaces_explain_last_with_initial_calculation_chain() {
    let mut entity = target();
    let attack = AttackRequest {
        source_id: "base:source".into(),
        target_id: "base:target".into(),
        accuracy_rating: 0,
        evasion_rating: 0,
        unavoidable: true,
        unparryable: true,
        parry: None,
    };
    let damage = request(DamageType::Physical, 10);
    let mut explanations = ExplanationStore::default();
    let mut rng = DeterministicRng::new(7);
    resolve_attack_action(
        &mut explanations,
        &mut rng,
        &ActionTraceContext {
            seed: 7,
            ruleset_version: "base:ruleset-1".into(),
            round: 2,
            action_id: "base:strike".into(),
        },
        &mut entity,
        &attack,
        &damage,
    )
    .unwrap();
    let last = explanations.last().unwrap();
    assert_eq!(
        (last.action_id.as_str(), last.declared_round, last.resolved),
        ("base:strike", 2, true)
    );
    assert_eq!(
        last.damage.as_ref().unwrap().stages[0].stage,
        "base_and_scaling"
    );
    assert!(explanations
        .render_json()
        .unwrap()
        .unwrap()
        .contains("tenacity_damage"));
}

fn request(kind: DamageType, amount: i64) -> DamageRequest {
    DamageRequest::from_scaling(
        "base:source",
        "base:target",
        kind,
        ScalingResult {
            flat_base_damage: amount,
            contributions: vec![],
            total: amount,
        },
    )
}

#[test]
fn physical_damage_follows_defense_shield_tenacity_hp_pipeline() {
    let mut entity = target();
    entity.defense = 10_000;
    entity.shield = 10;
    let result = resolve_damage(&mut entity, &request(DamageType::Physical, 100)).unwrap();
    assert_eq!(
        result
            .stages
            .iter()
            .map(|stage| stage.stage.as_str())
            .collect::<Vec<_>>(),
        vec![
            "base_and_scaling",
            "offensive_modifiers",
            "defensive_modifiers",
            "defense_and_damage_type",
            "block",
            "shield",
            "tenacity",
            "hp_modifiers",
            "hp"
        ]
    );
    assert_eq!(
        (
            result.shield_damage,
            result.tenacity_damage,
            result.hp_damage
        ),
        (10, 20, 20)
    );
    assert_eq!(
        (entity.shield, entity.tenacity.current, entity.hp.current),
        (0, 30, 80)
    );
}

#[test]
fn status_modifiers_feed_named_damage_stages() {
    let registry = StatusRegistry::base();
    let rules = AttributeRules::default();
    let mut source = Entity::new(
        StableId::new("base:source").unwrap(),
        "Source",
        Allegiance::Player,
        AttributeSet::default(),
        &rules,
    )
    .unwrap();
    let mut entity = target();
    entity.tenacity.current = 100;
    entity.tenacity.maximum = 100;
    registry
        .apply(
            &mut source.statuses,
            ActiveStatus::new("base:rage", "base:buff", 10, 2),
            0,
            &rules,
        )
        .unwrap();
    registry
        .apply(
            &mut entity.statuses,
            ActiveStatus::new("base:protect", "base:buff", 10, 2),
            0,
            &rules,
        )
        .unwrap();
    registry
        .apply(
            &mut entity.statuses,
            ActiveStatus::new("base:tremor", "base:debuff", 20, 2),
            0,
            &rules,
        )
        .unwrap();
    registry
        .apply(
            &mut entity.statuses,
            ActiveStatus::new("base:fragile", "base:debuff", 10, 2),
            0,
            &rules,
        )
        .unwrap();
    registry
        .apply(
            &mut entity.statuses,
            ActiveStatus::new("base:rupture", "base:debuff", 10, 2),
            0,
            &rules,
        )
        .unwrap();
    let mut damage = request(DamageType::Physical, 100);
    apply_status_modifiers_to_damage(&source, &entity, &registry, &mut damage).unwrap();
    let result = resolve_damage(&mut entity, &damage).unwrap();
    assert_eq!(result.stages[1].output, 110);
    assert_eq!(result.stages[2].output, 121);
    assert_eq!((result.tenacity_damage, result.hp_damage), (54, 73));
}

#[test]
fn true_damage_bypasses_defense_and_tenacity_but_not_shield() {
    let mut entity = target();
    entity.defense = 99_999;
    entity.shield = 7;
    let result = resolve_damage(&mut entity, &request(DamageType::True, 30)).unwrap();
    assert_eq!(
        (
            result.effective_defense,
            result.shield_damage,
            result.tenacity_damage,
            result.hp_damage
        ),
        (0, 7, 0, 23)
    );
    assert_eq!(entity.tenacity.current, 50);
}

#[test]
fn fully_shielded_damage_does_not_emit_on_damage_received() {
    let mut entity = target();
    entity.shield = 100;
    let result = resolve_damage(&mut entity, &request(DamageType::Magical, 20)).unwrap();
    assert!(result
        .events
        .iter()
        .any(|event| matches!(event, CombatEvent::OnShieldDamage { amount: 20, .. })));
    assert!(!result
        .events
        .iter()
        .any(|event| matches!(event, CombatEvent::OnDamageReceived { .. })));
}

#[test]
fn trigger_families_precede_hp_threshold_and_death_check() {
    let mut entity = target();
    entity.hp.current = 60;
    entity.tenacity.current = 0;
    let mut damage = request(DamageType::Physical, 70);
    damage.hp_thresholds = vec![50];
    let result = resolve_damage(&mut entity, &damage).unwrap();
    let names = result
        .events
        .iter()
        .map(|event| match event {
            CombatEvent::OnAttackReceived { .. } => "attack",
            CombatEvent::OnHpDamage { .. } => "hp",
            CombatEvent::OnDamageReceived { .. } => "damage",
            CombatEvent::OnHpBelow { .. } => "below",
            CombatEvent::OnKill { .. } => "kill",
            _ => "other",
        })
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["attack", "damage", "hp", "below", "kill"]);
}
