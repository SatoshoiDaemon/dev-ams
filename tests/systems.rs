use ams::{
    attributes::{
        Attribute, AttributeEnhancement, AttributeEnhancements, AttributeRules, AttributeSet,
        EnhancementResult,
    },
    effects::{ActiveStatus, CompetitionResult, StatusRegistry},
    events::{CombatEvent, EventBus, EventError, MAX_EVENTS_PER_ACTION},
    scaling::{calculate_scaling, ScalingGrade, ScalingSource},
};

#[test]
fn event_bus_enforces_trigger_depth_and_per_action_count() {
    let mut bus = EventBus::default();
    let event = || CombatEvent::TurnStarted {
        round: 1,
        entity_id: "base:actor".into(),
    };
    assert!(matches!(
        bus.publish(17, event()),
        Err(EventError::TriggerDepth { .. })
    ));
    bus.begin_action();
    for _ in 0..MAX_EVENTS_PER_ACTION {
        bus.publish(0, event()).unwrap();
    }
    assert!(matches!(
        bus.publish(0, event()),
        Err(EventError::EventCount { .. })
    ));
}

#[test]
fn attributes_derive_resources_power_and_resistance() {
    let mut attributes = AttributeSet::default();
    attributes.set(Attribute::Vigor, 12).unwrap();
    attributes.set(Attribute::Resistance, 5).unwrap();
    attributes.set(Attribute::Mana, 8).unwrap();
    attributes.set(Attribute::Strength, 10).unwrap();
    let rules = AttributeRules::default();
    let derived = rules.derive(&attributes).unwrap();
    assert_eq!(
        (derived.max_hp, derived.max_tenacity, derived.max_mana),
        (220, 100, 110)
    );
    assert_eq!(rules.power(&attributes, Attribute::Strength).unwrap(), 70);
    assert_eq!(rules.reduce_debuff_counter(10, 10).unwrap(), 9);
    assert!("spirit"
        .parse::<Attribute>()
        .is_ok_and(|attribute| attribute == Attribute::Intelligence));
}

#[test]
fn scaling_floors_each_contribution_before_summing() {
    let mut attributes = AttributeSet::default();
    attributes.set(Attribute::Strength, 10).unwrap();
    attributes.set(Attribute::Dexterity, 5).unwrap();
    let result = calculate_scaling(
        30,
        &attributes,
        &AttributeRules::default(),
        &[
            ScalingSource {
                source_id: "base:weapon-strength".into(),
                attribute: Attribute::Strength,
                grade: ScalingGrade::B,
            },
            ScalingSource {
                source_id: "base:weapon-dexterity".into(),
                attribute: Attribute::Dexterity,
                grade: ScalingGrade::C,
            },
        ],
    )
    .unwrap();
    assert_eq!(result.contributions[0].contribution, 52);
    assert_eq!(result.contributions[1].contribution, 17);
    assert_eq!(result.total, 99);
}

#[test]
fn attribute_enhancement_keeps_only_four_distinct_sources() {
    let mut enhancements = AttributeEnhancements::default();
    for index in 0..4 {
        assert_eq!(
            enhancements.add(
                Attribute::Strength,
                AttributeEnhancement {
                    source_id: format!("mod:source-{index}"),
                    amount: 2
                }
            ),
            EnhancementResult::Applied
        );
    }
    assert!(matches!(
        enhancements.add(
            Attribute::Strength,
            AttributeEnhancement {
                source_id: "mod:fifth".into(),
                amount: 99
            }
        ),
        EnhancementResult::IgnoredSourceLimit { .. }
    ));
    assert_eq!(enhancements.total(Attribute::Strength).unwrap(), 8);
}

#[test]
fn statuses_compete_by_potency_then_counter_after_resistance() {
    let registry = StatusRegistry::base();
    let rules = AttributeRules::default();
    let mut statuses = Default::default();
    let first = registry
        .apply(
            &mut statuses,
            ActiveStatus::new("base:burn", "base:first", 20, 30),
            0,
            &rules,
        )
        .unwrap();
    assert_eq!(first.competition, CompetitionResult::Applied);
    let stronger = registry
        .apply(
            &mut statuses,
            ActiveStatus::new("base:burn", "base:stronger", 25, 5),
            0,
            &rules,
        )
        .unwrap();
    assert!(matches!(
        stronger.competition,
        CompetitionResult::Replaced { .. }
    ));
    let longer = registry
        .apply(
            &mut statuses,
            ActiveStatus::new("base:burn", "base:longer", 25, 20),
            0,
            &rules,
        )
        .unwrap();
    assert!(matches!(
        longer.competition,
        CompetitionResult::Replaced { .. }
    ));
    let resisted = registry
        .apply(
            &mut statuses,
            ActiveStatus::new("base:poison", "base:resisted", 30, 10),
            10,
            &rules,
        )
        .unwrap();
    assert_eq!(resisted.counter_after_resistance, 9);
}

#[test]
fn status_decay_is_effect_specific_and_ordered_by_stable_id() {
    let registry = StatusRegistry::base();
    let rules = AttributeRules::default();
    let mut statuses = Default::default();
    registry
        .apply(
            &mut statuses,
            ActiveStatus::new("base:poison", "base:source", 10, 20),
            0,
            &rules,
        )
        .unwrap();
    registry
        .apply(
            &mut statuses,
            ActiveStatus::new("base:burn", "base:source", 30, 12),
            0,
            &rules,
        )
        .unwrap();
    let ticks = registry.end_turn_ticks(&mut statuses, 0).unwrap();
    assert_eq!(
        ticks
            .iter()
            .map(|tick| tick.effect_id.as_str())
            .collect::<Vec<_>>(),
        vec!["base:burn", "base:poison"]
    );
    assert_eq!(
        (
            ticks[0].damage,
            ticks[0].potency_after,
            ticks[0].counter_after
        ),
        (30, 29, 11)
    );
    assert_eq!((ticks[1].damage, ticks[1].counter_after), (10, 10));
    let second = registry.end_turn_ticks(&mut statuses, 0).unwrap();
    let poison = second
        .iter()
        .find(|tick| tick.effect_id == "base:poison")
        .unwrap();
    assert_eq!((poison.damage, poison.counter_after), (20, 5));
}
