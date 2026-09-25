use ams::{
    actions::{ActionRegistry, ActionRejection, BonusActionTracker},
    combat::{ActionTraceContext, CombatOutcome, DeterministicRng, ExplanationStore},
    config::NumericConfig,
    effects::{ActiveStatus, StatusRegistry},
    encounters::EncounterRegistry,
    lua::{LuaWorldSnapshot, ModRuntime},
    runtime::{
        confirm_pending_deaths, process_end_round, process_queued_events, resolve_declared_action,
    },
    saves::{read_save, write_save, SaveData, SaveMetadata, SAVE_FORMAT_VERSION},
    session::{CombatPhase, CombatSession, ReactionReservation},
};
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf};

fn content() -> (ActionRegistry, EncounterRegistry, StatusRegistry) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/content");
    (
        ActionRegistry::load(root.join("actions.json")).unwrap(),
        EncounterRegistry::load(&root).unwrap(),
        StatusRegistry::load(root.join("statuses.json")).unwrap(),
    )
}

fn resolve_round(
    session: &mut CombatSession,
    actions: &ActionRegistry,
    statuses: &StatusRegistry,
    rng: &mut DeterministicRng,
    explanations: &mut ExplanationStore,
) {
    let rules = NumericConfig::default().attribute_rules();
    session.close_declarations();
    while let Some(declaration) = session.next_action() {
        let _ = resolve_declared_action(
            session,
            actions.as_map(),
            &rules,
            statuses,
            rng,
            explanations,
            &declaration,
            &ActionTraceContext {
                seed: 7,
                ruleset_version: "base:standard@1".into(),
                round: declaration.declared_round,
                action_id: declaration.action_id.clone(),
            },
        );
        confirm_pending_deaths(session).unwrap();
        session.update_outcome();
    }
}

#[test]
fn training_encounter_runs_to_victory_and_flee_is_immediate() {
    let (actions, encounters, statuses) = content();
    let rules = NumericConfig::default().attribute_rules();
    let mut session = encounters
        .build_session("base:training-encounter", &rules)
        .unwrap();
    let mut rng = DeterministicRng::new(7);
    let mut explanations = ExplanationStore::default();
    session.start_round().unwrap();
    process_queued_events(&mut session, &statuses, &rules).unwrap();

    for _ in 0..20 {
        if session.outcome.is_some() {
            break;
        }
        let target = session
            .entities
            .values()
            .find(|entity| {
                entity.allegiance == ams::entities::Allegiance::Enemy && entity.is_alive()
            })
            .unwrap()
            .id
            .as_str()
            .to_owned();
        let action = if session.entities["base:player"].mana.current >= 12 {
            actions.get("base:black-flame-orb-demo").unwrap()
        } else {
            actions.get("base:unarmed-strike").unwrap()
        };
        session
            .submit_action("base:player", action, &[target], &rules)
            .unwrap();
        let enemy_ids = session
            .entities
            .values()
            .filter(|entity| {
                entity.allegiance == ams::entities::Allegiance::Enemy && entity.is_alive()
            })
            .map(|entity| entity.id.as_str().to_owned())
            .collect::<Vec<_>>();
        for enemy in enemy_ids {
            session
                .submit_action(
                    &enemy,
                    actions.get("base:training-strike").unwrap(),
                    &["base:player".into()],
                    &rules,
                )
                .unwrap();
        }
        resolve_round(
            &mut session,
            &actions,
            &statuses,
            &mut rng,
            &mut explanations,
        );
        if session.outcome.is_none() {
            process_end_round(&mut session, &statuses, &rules).unwrap();
            confirm_pending_deaths(&mut session).unwrap();
            session.update_outcome();
        }
        if session.outcome.is_none() {
            session.start_round().unwrap();
            process_queued_events(&mut session, &statuses, &rules).unwrap();
        }
    }
    assert_eq!(session.outcome, Some(CombatOutcome::Victory));
    assert!(explanations.last().is_some());

    let mut fled = encounters
        .build_session("base:training-encounter", &rules)
        .unwrap();
    fled.start_round().unwrap();
    fled.flee();
    assert_eq!(fled.outcome, Some(CombatOutcome::Fled));
    assert_eq!(fled.phase, CombatPhase::Complete);
}

#[test]
fn reaction_and_bonus_action_limits_preserve_atomic_budgets() {
    let (actions, encounters, _) = content();
    let rules = NumericConfig::default().attribute_rules();
    let mut session = encounters
        .build_session("base:training-encounter", &rules)
        .unwrap();
    session.start_round().unwrap();
    session
        .reserve_reaction("base:player", actions.get("base:guard").unwrap())
        .unwrap();
    assert_eq!(session.budgets.available("base:player"), 50);
    assert!(matches!(
        session.reactions.get("base:player"),
        Some(ReactionReservation::Block { power: 20, .. })
    ));
    let before = session.budgets.clone();
    assert_eq!(
        session.reserve_reaction("base:player", actions.get("base:parry").unwrap()),
        Err(ActionRejection::ReactionAlreadyReserved)
    );
    assert_eq!(session.budgets, before);

    let mut tracker = BonusActionTracker::default();
    tracker.register("base:player", 1, "test:one", 1).unwrap();
    assert_eq!(
        tracker.register("base:player", 1, "test:one", 1),
        Err(ActionRejection::RepeatedBonusSource)
    );
    tracker.register("base:player", 1, "test:two", 2).unwrap();
    tracker.register("base:player", 1, "test:three", 2).unwrap();
    assert_eq!(
        tracker.register("base:player", 1, "test:four", 2),
        Err(ActionRejection::BonusLimitExceeded)
    );
    let mut depth = BonusActionTracker::default();
    assert_eq!(
        depth.register("base:player", 2, "test:deep", 3),
        Err(ActionRejection::BonusDepthExceeded)
    );

    let mut insertion = encounters
        .build_session("base:training-encounter", &rules)
        .unwrap();
    insertion.start_round().unwrap();
    insertion.close_declarations();
    let mut bonus = actions.get("base:unarmed-strike").unwrap().clone();
    bonus.properties.bonus_eligible = true;
    let declaration = insertion
        .insert_bonus_action(
            "base:player",
            &bonus,
            &["base:raider-1".into()],
            "test:bonus",
            1,
            1,
            &rules,
        )
        .unwrap();
    assert_eq!(declaration.ap_cost, 50);
    assert_eq!(insertion.eligible_queue.front(), Some(&declaration));
}

#[test]
fn active_combat_save_round_trip_preserves_cast_status_rng_and_explanation() {
    let (actions, encounters, statuses) = content();
    let rules = NumericConfig::default().attribute_rules();
    let mut session = encounters
        .build_session("base:training-encounter", &rules)
        .unwrap();
    session.start_round().unwrap();
    let mut cast = actions.get("base:black-flame-orb-demo").unwrap().clone();
    cast.cast = 1;
    session
        .submit_action("base:player", &cast, &["base:raider-1".into()], &rules)
        .unwrap();
    session
        .entities
        .get_mut("base:raider-1")
        .unwrap()
        .statuses
        .set_runtime(ActiveStatus::new("base:dark-flame", "base:player", 10, 3));
    let explanation = ams::combat::ExplainLast {
        ruleset_version: "base:standard@1".into(),
        rng_seed: 7,
        rng_state_before: 7,
        rng_state_after: 7,
        actor_id: "base:player".into(),
        action_id: "base:black-flame-orb-demo".into(),
        target_ids: vec!["base:raider-1".into()],
        declared_round: 1,
        resolved: false,
        rejection_reason: None,
        payment_ap: 100,
        payment_mana: 12,
        eligible_round: 2,
        queue_position: None,
        action_stages: Vec::new(),
        trigger_order: Vec::new(),
        attack: None,
        damage: None,
        events: Vec::new(),
    };
    session.last_explanation = Some(explanation);
    let save = SaveData {
        metadata: SaveMetadata {
            save_format_version: SAVE_FORMAT_VERSION,
            ruleset_version: "base:standard@1".into(),
            required_mods: Vec::new(),
            gamemode_id: "base:standard".into(),
            player_entity_ids: vec!["base:player".into()],
            rng_state: 99,
        },
        ruleset: json!({"attribute_rules": rules}),
        entities: session.entities.values().cloned().collect(),
        combat: Some(session.clone()),
        opaque_mod_data: BTreeMap::from([("missing:mod".into(), json!({"kept": true}))]),
        migration_warnings: Vec::new(),
    };
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("cast.zip");
    write_save(&path, &save).unwrap();
    let loaded = read_save(&path).unwrap();
    assert_eq!(loaded, save);
    let combat = loaded.combat.unwrap();
    assert_eq!(combat.pending_casts.len(), 1);
    assert!(combat.entities["base:raider-1"]
        .statuses
        .get("base:dark-flame")
        .is_some());
    assert_eq!(combat.last_explanation, session.last_explanation);
    assert_eq!(loaded.opaque_mod_data["missing:mod"]["kept"], true);
    assert!(statuses.get("base:dark-flame").is_some());
}

#[test]
fn lua_callbacks_are_namespaced_bounded_isolated_and_command_only() {
    let runtime = ModRuntime::new("test:mod", 10_000).unwrap();
    runtime
        .load_script(
            "test.lua",
            r#"
              game.on("OnHPDamage", "test:good", function(event)
                game.heal(event.target_id, 2)
              end)
              game.on("OnKill", "test:bad", function(_) error("intentional") end)
            "#,
        )
        .unwrap();
    runtime.finish_loading();
    runtime.set_world_snapshot(LuaWorldSnapshot::default());
    let called = runtime
        .dispatch_event("OnHPDamage", &json!({"target_id": "base:player"}))
        .unwrap();
    assert_eq!(called, vec!["test:good"]);
    assert_eq!(runtime.drain_commands().len(), 1);
    assert!(runtime.dispatch_event("OnKill", &json!({})).is_err());
    assert!(runtime
        .dispatch_event("OnKill", &json!({}))
        .unwrap()
        .is_empty());
    assert!(runtime
        .load_script("late.lua", "game.register_action({id='test:late'})")
        .is_err());
}

#[test]
fn demon_half_hp_crossing_uses_exact_integer_boundary() {
    let (_, encounters, statuses) = content();
    let rules = NumericConfig::default().attribute_rules();
    let mut session = encounters
        .build_session("base:training-encounter", &rules)
        .unwrap();
    session.entities.get_mut("base:player").unwrap().hp.current = 100;
    session
        .events
        .publish(
            0,
            ams::events::CombatEvent::OnHpBelow {
                target_id: "base:player".into(),
                threshold_percent: 50,
            },
        )
        .unwrap();
    process_queued_events(&mut session, &statuses, &rules).unwrap();
    assert_eq!(session.demon_states["base:player"].hearts, 7);
    assert_eq!(session.entities["base:player"].shield, 0);

    session.entities.get_mut("base:player").unwrap().hp.current = 99;
    session
        .events
        .publish(
            0,
            ams::events::CombatEvent::OnHpBelow {
                target_id: "base:player".into(),
                threshold_percent: 50,
            },
        )
        .unwrap();
    process_queued_events(&mut session, &statuses, &rules).unwrap();
    assert_eq!(session.demon_states["base:player"].hearts, 6);
    assert_eq!(session.entities["base:player"].shield, 40);
    assert_eq!(
        session.entities["base:player"]
            .statuses
            .get("base:broken-heart")
            .unwrap()
            .counter,
        2
    );
}

#[test]
fn cast_becomes_eligible_next_round_and_silence_cancels_without_refund() {
    let (actions, encounters, statuses) = content();
    let rules = NumericConfig::default().attribute_rules();
    let mut session = encounters
        .build_session("base:training-encounter", &rules)
        .unwrap();
    session.start_round().unwrap();
    let mut cast = actions.get("base:black-flame-orb-demo").unwrap().clone();
    cast.cast = 1;
    let mana_before = session.entities["base:player"].mana.current;
    session
        .submit_action("base:player", &cast, &["base:raider-1".into()], &rules)
        .unwrap();
    assert_eq!(session.pending_casts[0].declaration.eligible_round, 2);
    assert_eq!(
        session.entities["base:player"].mana.current,
        mana_before - 12
    );
    session.close_declarations();
    assert!(session.next_action().is_none());
    session.finish_round().unwrap();
    session.start_round().unwrap();
    statuses
        .apply(
            &mut session.entities.get_mut("base:player").unwrap().statuses,
            ActiveStatus::new("base:silent", "base:raider-1", 1, 2),
            0,
            &rules,
        )
        .unwrap();
    session.close_declarations();
    let declaration = session.next_action().unwrap();
    let error = resolve_declared_action(
        &mut session,
        actions.as_map(),
        &rules,
        &statuses,
        &mut DeterministicRng::new(7),
        &mut ExplanationStore::default(),
        &declaration,
        &ActionTraceContext {
            seed: 7,
            ruleset_version: "base:standard@1".into(),
            round: 2,
            action_id: declaration.action_id.clone(),
        },
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ams::runtime::RuntimeError::Action(ActionRejection::CastCancelled)
    ));
    assert_eq!(
        session.entities["base:player"].mana.current,
        mana_before - 12
    );
}
