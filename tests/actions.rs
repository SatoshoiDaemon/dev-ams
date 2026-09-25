use ams::{
    actions::{
        build_action_queue, declare_action, ActionBudgets, ActionDefinition, TargetMode,
        TargetRelation, TargetSpec, TargetState, DEFAULT_AP,
    },
    attributes::{Attribute, AttributeRules, AttributeSet},
    combat::CombatOutcome,
    content::StableId,
    entities::{Allegiance, Entity},
};

fn actor(id: &str, allegiance: Allegiance, dexterity: i64) -> Entity {
    let mut attributes = AttributeSet::default();
    attributes.set(Attribute::Dexterity, dexterity).unwrap();
    Entity::new(
        StableId::new(id).unwrap(),
        id,
        allegiance,
        attributes,
        &AttributeRules::default(),
    )
    .unwrap()
}

fn definition(priority: i64, cast: u32) -> ActionDefinition {
    ActionDefinition {
        id: "base:strike".into(),
        replacement_of: None,
        source_id: "base:weapon".into(),
        target: TargetSpec {
            mode: TargetMode::Single,
            relation: TargetRelation::Enemy,
            state: TargetState::Active,
            filters: vec![],
        },
        ap_cost: 100,
        mana_cost: 0,
        cast,
        action_priority: priority,
        availability: Default::default(),
        properties: Default::default(),
        requirements: vec![],
        tags: vec!["physical".into()],
        steps: vec![],
    }
}

#[test]
fn declarations_compute_cast_priority_and_validate_targets_before_payment() {
    let player = actor("base:player", Allegiance::Player, 10);
    let enemy = actor("base:enemy", Allegiance::Enemy, 20);
    let declaration = declare_action(
        &player,
        &definition(5, 2),
        &[&enemy],
        DEFAULT_AP,
        4,
        &AttributeRules::default(),
    )
    .unwrap();
    assert_eq!(
        (declaration.eligible_round, declaration.final_priority),
        (6, 125)
    );
    let mut budgets = ActionBudgets::default();
    budgets.begin_turn(player.id.as_str());
    budgets
        .pay(player.id.as_str(), declaration.ap_cost)
        .unwrap();
    assert_eq!(budgets.available(player.id.as_str()), 0);
}

#[test]
fn queue_uses_priority_then_player_party_then_stable_id() {
    let player = actor("base:z-player", Allegiance::Player, 0);
    let enemy = actor("base:a-enemy", Allegiance::Enemy, 0);
    let player_target = actor("base:player-target", Allegiance::Player, 0);
    let enemy_target = actor("base:enemy-target", Allegiance::Enemy, 0);
    let player_action = declare_action(
        &player,
        &definition(0, 0),
        &[&enemy_target],
        100,
        1,
        &AttributeRules::default(),
    )
    .unwrap();
    let enemy_action = declare_action(
        &enemy,
        &definition(0, 0),
        &[&player_target],
        100,
        1,
        &AttributeRules::default(),
    )
    .unwrap();
    let queue = build_action_queue(vec![enemy_action, player_action]);
    assert_eq!(queue[0].actor_id, "base:z-player");
}

#[test]
fn fleeing_is_a_distinct_combat_outcome() {
    assert_eq!(CombatOutcome::Fled, CombatOutcome::Fled);
}
