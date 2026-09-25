use crate::{
    actions::{ActionDefinition, TargetMode},
    combat::{explain_rejection, ActionTraceContext, DeterministicRng, RejectionTrace},
    engine::GameState,
    logging::DiagnosticContext,
    lua::{LuaWorldSnapshot, ModRuntime},
    runtime::{
        confirm_pending_deaths, process_end_round, process_queued_events, resolve_declared_action,
    },
    saves::{
        read_save, validate_save_name, write_save, RequiredMod, SaveData, SaveMetadata,
        SAVE_FORMAT_VERSION,
    },
    terminal::{PlainTerminal, TerminalRenderer},
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs,
    io::{BufRead, Write},
    path::Path,
    sync::mpsc,
};

enum InputEvent {
    Line(String),
    Interrupt,
    Closed,
    Error(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppMode {
    Menu,
    Ready,
    Combat,
}

pub struct App {
    pub state: GameState,
    terminal: PlainTerminal,
    pub mode: AppMode,
    pub verbose: bool,
    pub should_exit: bool,
    pub dirty: bool,
    save_name: Option<String>,
    gamemode_id: String,
    pending_load: Option<String>,
    pending_quit: bool,
    lua_runtimes: Vec<ModRuntime>,
}

impl App {
    pub fn new(mut state: GameState) -> Self {
        let lua_runtimes = load_mod_extensions(&mut state);
        Self {
            state,
            terminal: PlainTerminal,
            mode: AppMode::Menu,
            verbose: false,
            should_exit: false,
            dirty: false,
            save_name: None,
            gamemode_id: "base:standard".into(),
            pending_load: None,
            pending_quit: false,
            lua_runtimes,
        }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        let (sender, receiver) = mpsc::channel();
        let input_sender = sender.clone();
        std::thread::spawn(move || {
            let stdin = std::io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(line) => {
                        if input_sender.send(InputEvent::Line(line)).is_err() {
                            return;
                        }
                    }
                    Err(error) => {
                        let _ = input_sender.send(InputEvent::Error(error.to_string()));
                        return;
                    }
                }
            }
            let _ = input_sender.send(InputEvent::Closed);
        });
        ctrlc::set_handler(move || {
            let _ = sender.send(InputEvent::Interrupt);
        })
        .map_err(std::io::Error::other)?;
        self.render_message("A Magic Sovereign — type 'help' for commands.");
        while !self.should_exit {
            print!("> ");
            std::io::stdout().flush()?;
            match receiver.recv().map_err(std::io::Error::other)? {
                InputEvent::Line(input) => {
                    let response = self.command(&input);
                    if !response.is_empty() {
                        self.render_message(&response);
                    }
                }
                InputEvent::Interrupt if self.dirty => {
                    self.render_message(
                        "Ctrl+C received with unsaved state. Type 'save', then 'quit', or use 'confirm quit' to discard it.",
                    );
                }
                InputEvent::Interrupt => {
                    self.should_exit = true;
                    self.render_message("Ctrl+C received. Goodbye.");
                }
                InputEvent::Closed => {
                    if self.dirty {
                        self.render_message("Input closed with unsaved state. Use 'save' before closing the terminal when possible.");
                    }
                    break;
                }
                InputEvent::Error(error) => return Err(std::io::Error::other(error)),
            }
        }
        Ok(())
    }

    pub fn render_message(&mut self, message: &str) {
        self.terminal.render(message);
    }

    pub fn command(&mut self, input: &str) -> String {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        let parts = trimmed.split_whitespace().collect::<Vec<_>>();
        if parts.first() == Some(&"confirm") {
            return self.confirm(&parts[1..]);
        }
        match parts[0] {
            "help" => self.help(),
            "new" => self.new_game(&parts[1..]),
            "load" => self.request_load(&parts[1..]),
            "saves" => self.list_saves(),
            "status" => self.status(),
            "combat" => self.start_combat(&parts[1..]),
            "actions" => self.actions(),
            "use" => self.use_action(&parts[1..]),
            "reserve" => self.reserve(&parts[1..]),
            "end" => self.end_round(),
            "save" => self.save(),
            "explain" if parts.get(1) == Some(&"last") => self
                .state
                .explanations
                .render_json()
                .map(|value| value.unwrap_or_else(|| "No completed action to explain.".into()))
                .unwrap_or_else(|error| format!("Failed to render explain last: {error}")),
            "log" => self.set_log_mode(&parts[1..]),
            "flee" => self.flee(),
            "menu" => {
                self.mode = AppMode::Menu;
                "Returned to menu.".into()
            }
            "quit" => self.request_quit(),
            "cancel" => {
                self.pending_load = None;
                self.pending_quit = false;
                "Pending operation cancelled.".into()
            }
            command => format!("Unknown command: {command}. Type 'help'."),
        }
    }

    fn help(&self) -> String {
        match self.mode {
            AppMode::Menu => "Commands: new <save> [gamemode], load <save>, saves, help, quit".into(),
            AppMode::Ready => "Commands: status, combat base:training-encounter, save, explain last, log normal|verbose, menu, quit".into(),
            AppMode::Combat => "Commands: status, actions, use <action-id> [target-id...], reserve <reaction-id>, end, save, explain last, log normal|verbose, flee, quit".into(),
        }
    }

    fn new_game(&mut self, arguments: &[&str]) -> String {
        let Some(name) = arguments.first() else {
            return "Usage: new <save-name> [gamemode-id]".into();
        };
        if let Err(error) = validate_save_name(name) {
            return error.to_string();
        }
        let gamemode = arguments.get(1).copied().unwrap_or("base:standard");
        let Some(mode) = self.state.game_modes.get(gamemode) else {
            return format!("Unknown game mode {gamemode}");
        };
        self.state.attribute_rules = mode
            .resolve_numeric(&self.state.global_numeric)
            .attribute_rules();
        self.state.ruleset_version = mode.ruleset_version.clone();
        self.state.combat = None;
        self.state.rng = DeterministicRng::new(self.state.seed);
        self.save_name = Some((*name).into());
        self.gamemode_id = gamemode.into();
        self.mode = AppMode::Ready;
        self.dirty = true;
        format!("Created Demon save '{name}' using {gamemode}.")
    }

    fn request_load(&mut self, arguments: &[&str]) -> String {
        let Some(name) = arguments.first() else {
            return "Usage: load <save-name>".into();
        };
        if self.dirty {
            self.pending_load = Some((*name).into());
            return "Unsaved state will be replaced. Type 'confirm load' or 'cancel'.".to_owned();
        }
        self.load_named(name)
    }

    fn confirm(&mut self, arguments: &[&str]) -> String {
        match arguments.first().copied() {
            Some("load") => {
                let Some(name) = self.pending_load.take() else {
                    return "No load operation is pending.".into();
                };
                self.load_named(&name)
            }
            Some("quit") if self.pending_quit => {
                self.should_exit = true;
                self.pending_quit = false;
                "Exiting without saving.".into()
            }
            _ => "Nothing matching that confirmation is pending.".into(),
        }
    }

    fn load_named(&mut self, name: &str) -> String {
        if let Err(error) = validate_save_name(name) {
            return error.to_string();
        }
        let Some(paths) = &self.state.paths else {
            return "Runtime paths are unavailable.".into();
        };
        let path = paths.saves.join(format!("{name}.zip"));
        self.set_diagnostic_context(DiagnosticContext {
            phase: Some("save_load".into()),
            save: Some(name.into()),
            file: Some(path.display().to_string()),
            ..DiagnosticContext::default()
        });
        match read_save(&path) {
            Ok(save) => {
                let migration_warnings = save.migration_warnings.clone();
                if let Some(snapshot) = save.ruleset.get("attribute_rules") {
                    match serde_json::from_value(snapshot.clone()) {
                        Ok(rules) => self.state.attribute_rules = rules,
                        Err(error) => {
                            return format!(
                                "Failed to load ruleset snapshot from {}: {error}",
                                path.display()
                            )
                        }
                    }
                } else if let Some(mode) = self.state.game_modes.get(&save.metadata.gamemode_id) {
                    self.state.attribute_rules = mode
                        .resolve_numeric(&self.state.global_numeric)
                        .attribute_rules();
                }
                self.state.rng = DeterministicRng::new(save.metadata.rng_state);
                self.state.ruleset_version = save.metadata.ruleset_version.clone();
                self.gamemode_id = save.metadata.gamemode_id;
                self.state.combat = save.combat;
                if let Some(explanation) = self
                    .state
                    .combat
                    .as_ref()
                    .and_then(|combat| combat.last_explanation.clone())
                {
                    self.state.explanations.replace(explanation);
                }
                self.mode = if self.state.combat.is_some() {
                    AppMode::Combat
                } else {
                    AppMode::Ready
                };
                self.save_name = Some(name.into());
                self.dirty = false;
                if migration_warnings.is_empty() {
                    format!("Loaded save {}.", path.display())
                } else {
                    format!(
                        "Loaded save {}.\nWarning: {}",
                        path.display(),
                        migration_warnings.join("\nWarning: ")
                    )
                }
            }
            Err(error) => format!("Failed to load save {}: {error}", path.display()),
        }
    }

    fn list_saves(&self) -> String {
        let Some(paths) = &self.state.paths else {
            return "Runtime paths are unavailable.".into();
        };
        let mut saves = fs::read_dir(&paths.saves)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("zip"))
            .filter_map(|path| {
                path.file_stem()
                    .map(|name| name.to_string_lossy().into_owned())
            })
            .collect::<Vec<_>>();
        saves.sort();
        if saves.is_empty() {
            "No saves found.".into()
        } else {
            format!("Saves:\n{}", saves.join("\n"))
        }
    }

    fn start_combat(&mut self, arguments: &[&str]) -> String {
        if self.mode != AppMode::Ready {
            return "Combat can only start from the ready state.".into();
        }
        let encounter = arguments
            .first()
            .copied()
            .unwrap_or("base:training-encounter");
        self.set_diagnostic_context(DiagnosticContext {
            phase: Some("combat_start".into()),
            action_id: Some(encounter.into()),
            ..DiagnosticContext::default()
        });
        match self
            .state
            .encounter_registry
            .build_session(encounter, &self.state.attribute_rules)
        {
            Ok(mut session) => {
                if let Err(error) = session.start_round() {
                    return format!("Failed to start combat: {error}");
                }
                if let Err(error) = process_queued_events(
                    &mut session,
                    &self.state.status_registry,
                    &self.state.attribute_rules,
                ) {
                    return format!("Failed to process combat opening: {error}");
                }
                self.dispatch_lua_events(&mut session, 0);
                self.state.combat = Some(session);
                self.mode = AppMode::Combat;
                self.dirty = true;
                format!("Combat started: {encounter}. Round 1 declaration.")
            }
            Err(error) => format!("Failed to start combat: {error}"),
        }
    }

    fn status(&self) -> String {
        let Some(session) = &self.state.combat else {
            return format!(
                "Ready. Save: {}. Game mode: {}. Ruleset: {}.",
                self.save_name.as_deref().unwrap_or("none"),
                self.gamemode_id,
                self.state.ruleset_version
            );
        };
        let mut lines = vec![format!(
            "Encounter {} — round {} — {:?}",
            session.encounter_id, session.round, session.phase
        )];
        for entity in session.entities.values() {
            lines.push(format!(
                "{} [{}] HP {}/{} | Tenacity {}/{} | Mana {}/{} | Shield {}",
                entity.name,
                entity.id,
                entity.hp.current,
                entity.hp.maximum,
                entity.tenacity.current,
                entity.tenacity.maximum,
                entity.mana.current,
                entity.mana.maximum,
                entity.shield
            ));
        }
        if let Some(outcome) = &session.outcome {
            lines.push(format!("Outcome: {outcome:?}"));
        }
        lines.join("\n")
    }

    fn actions(&self) -> String {
        let Some(session) = &self.state.combat else {
            return "No combat is active.".into();
        };
        let Some(action_ids) = session.available_actions.get("base:player") else {
            return "The player has no active actions.".into();
        };
        let available = session.budgets.available("base:player");
        let mut lines = vec![format!("Available AP: {available}")];
        let targets = session
            .entities
            .values()
            .filter(|entity| {
                entity.allegiance == crate::entities::Allegiance::Enemy && entity.is_alive()
            })
            .map(|entity| format!("{} ({})", entity.id, entity.name))
            .collect::<Vec<_>>();
        if !targets.is_empty() {
            lines.push(format!("Enemy targets: {}", targets.join(", ")));
        }
        for id in action_ids {
            if let Some(action) = self.state.action_registry.get(id) {
                lines.push(format!(
                    "{} — {} AP, {} Mana, Cast {}",
                    action.id, action.ap_cost, action.mana_cost, action.cast
                ));
            }
        }
        lines.join("\n")
    }

    fn use_action(&mut self, arguments: &[&str]) -> String {
        let Some(action_id) = arguments.first() else {
            return "Usage: use <action-id> [target-id...]".into();
        };
        let Some(definition) = self.state.action_registry.get(action_id).cloned() else {
            return format!("Unknown action {action_id}");
        };
        let Some(session) = self.state.combat.as_mut() else {
            return "No combat is active.".into();
        };
        if !session
            .available_actions
            .get("base:player")
            .is_some_and(|ids| ids.iter().any(|id| id == action_id))
        {
            return format!("Action {action_id} is not available to base:player");
        }
        if definition.tags.iter().any(|tag| tag == "reaction") {
            return format!("Use 'reserve {action_id}' for reactions.");
        }
        if action_id == &"base:flee" {
            return self.flee();
        }
        let mut target_ids = arguments[1..]
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>();
        if definition.target.mode == TargetMode::SelfOnly && target_ids.is_empty() {
            target_ids.push("base:player".into());
        }
        match session.submit_action(
            "base:player",
            &definition,
            &target_ids,
            &self.state.attribute_rules,
        ) {
            Ok(declaration) => {
                self.dirty = true;
                format!(
                    "Declared {} for round {}; eligible round {}. Remaining AP: {}.",
                    declaration.action_id,
                    declaration.declared_round,
                    declaration.eligible_round,
                    session.budgets.available("base:player")
                )
            }
            Err(error) => {
                let trace = RejectionTrace {
                    context: ActionTraceContext {
                        seed: self.state.seed,
                        ruleset_version: self.state.ruleset_version.clone(),
                        round: session.round,
                        action_id: definition.id.clone(),
                    },
                    rng_state: self.state.rng.state(),
                    actor_id: "base:player".into(),
                    target_ids,
                    reason: format!("{error:?}"),
                };
                explain_rejection(&mut self.state.explanations, trace);
                session.last_explanation = self.state.explanations.last().cloned();
                format!("Action rejected: {error:?}")
            }
        }
    }

    fn reserve(&mut self, arguments: &[&str]) -> String {
        let Some(reaction_id) = arguments.first() else {
            return "Usage: reserve <reaction-id>".into();
        };
        let Some(definition) = self.state.action_registry.get(reaction_id).cloned() else {
            return format!("Unknown reaction {reaction_id}");
        };
        let Some(session) = self.state.combat.as_mut() else {
            return "No combat is active.".into();
        };
        match session.reserve_reaction("base:player", &definition) {
            Ok(()) => {
                self.dirty = true;
                format!("Reserved 50 AP for {reaction_id}.")
            }
            Err(error) => format!("Reaction rejected: {error:?}"),
        }
    }

    fn end_round(&mut self) -> String {
        let Some(mut session) = self.state.combat.take() else {
            return "No combat is active.".into();
        };
        let player_targets = session
            .entities
            .values()
            .filter(|entity| {
                entity.allegiance == crate::entities::Allegiance::Player && entity.is_alive()
            })
            .map(|entity| entity.id.as_str().to_owned())
            .collect::<Vec<_>>();
        let enemies = session
            .entities
            .values()
            .filter(|entity| {
                entity.allegiance == crate::entities::Allegiance::Enemy && entity.is_alive()
            })
            .map(|entity| entity.id.as_str().to_owned())
            .collect::<Vec<_>>();
        for enemy_id in enemies {
            let Some(action_id) = session
                .available_actions
                .get(&enemy_id)
                .and_then(|ids| ids.first())
                .cloned()
            else {
                continue;
            };
            let Some(definition) = self.state.action_registry.get(&action_id) else {
                continue;
            };
            if let Some(target) = player_targets.first() {
                let _ = session.submit_action(
                    &enemy_id,
                    definition,
                    std::slice::from_ref(target),
                    &self.state.attribute_rules,
                );
            }
        }
        session.close_declarations();
        let mut messages = Vec::new();
        while let Some(declaration) = session.next_action() {
            self.set_diagnostic_context(DiagnosticContext {
                phase: Some("combat_resolution".into()),
                entity_id: Some(declaration.actor_id.clone()),
                action_id: Some(declaration.action_id.clone()),
                ..DiagnosticContext::default()
            });
            let context = ActionTraceContext {
                seed: self.state.seed,
                ruleset_version: self.state.ruleset_version.clone(),
                round: declaration.declared_round,
                action_id: declaration.action_id.clone(),
            };
            match resolve_declared_action(
                &mut session,
                self.state.action_registry.as_map(),
                &self.state.attribute_rules,
                &self.state.status_registry,
                &mut self.state.rng,
                &mut self.state.explanations,
                &declaration,
                &context,
            ) {
                Ok(result) => {
                    self.dispatch_lua_events(&mut session, 0);
                    let death_cursor = session.processed_events.len();
                    match confirm_pending_deaths(&mut session) {
                        Ok(_) => self.dispatch_lua_events(&mut session, death_cursor),
                        Err(error) => messages.push(format!("Death check failed: {error}")),
                    }
                    session.update_outcome();
                    let damage = result
                        .damage_results
                        .iter()
                        .map(|damage| {
                            damage.hp_damage + damage.tenacity_damage + damage.shield_damage
                        })
                        .sum::<i64>();
                    messages.push(format!(
                        "{} resolved ({} total layer damage).",
                        result.action_id, damage
                    ));
                    if let Some(logger) = &self.state.logger {
                        let _ = logger.info(&format!(
                            "Action {} by {} resolved against {:?}; total layer damage {}",
                            result.action_id, result.actor_id, result.target_ids, damage
                        ));
                        if let Ok(Some(explanation)) = self.state.explanations.render_json() {
                            let _ = logger.trace("combat", &explanation);
                        }
                    }
                    if self.verbose {
                        messages.extend(result.stages.into_iter().map(|stage| {
                            format!(
                                "  {}: {} -> {} ({})",
                                stage.stage, stage.input, stage.output, stage.detail
                            )
                        }));
                    }
                }
                Err(error) => {
                    explain_rejection(
                        &mut self.state.explanations,
                        RejectionTrace {
                            context,
                            rng_state: self.state.rng.state(),
                            actor_id: declaration.actor_id.clone(),
                            target_ids: declaration.target_ids.clone(),
                            reason: error.to_string(),
                        },
                    );
                    session.last_explanation = self.state.explanations.last().cloned();
                    if let Some(logger) = &self.state.logger {
                        let _ = logger.warn(&format!(
                            "Action {} by {} was rejected: {error}",
                            declaration.action_id, declaration.actor_id
                        ));
                    }
                    messages.push(format!("{} rejected: {error}", declaration.action_id));
                }
            }
            if session.outcome.is_some() {
                break;
            }
        }
        if session.outcome.is_none() {
            if let Err(error) = process_end_round(
                &mut session,
                &self.state.status_registry,
                &self.state.attribute_rules,
            ) {
                messages.push(format!("End-of-round processing failed: {error}"));
            }
            if let Err(error) = session.finish_round() {
                messages.push(format!("Round completion failed: {error}"));
            }
            self.dispatch_lua_events(&mut session, 0);
            let death_cursor = session.processed_events.len();
            match confirm_pending_deaths(&mut session) {
                Ok(_) => self.dispatch_lua_events(&mut session, death_cursor),
                Err(error) => messages.push(format!("Death check failed: {error}")),
            }
            session.update_outcome();
        }
        if session.outcome.is_none() {
            if let Err(error) = session.start_round() {
                messages.push(format!("Failed to start next round: {error}"));
            } else if let Err(error) = process_queued_events(
                &mut session,
                &self.state.status_registry,
                &self.state.attribute_rules,
            ) {
                messages.push(format!("Turn-start processing failed: {error}"));
            } else {
                self.dispatch_lua_events(&mut session, 0);
            }
        }
        if let Some(outcome) = &session.outcome {
            messages.push(format!("Combat completed: {outcome:?}."));
            self.mode = AppMode::Ready;
        }
        self.state.combat = Some(session);
        self.dirty = true;
        messages.join("\n")
    }

    fn save(&mut self) -> String {
        let Some(name) = self.save_name.as_deref() else {
            return "Create or load a named save first.".into();
        };
        let Some(paths) = &self.state.paths else {
            return "Runtime paths are unavailable.".into();
        };
        let entities = self
            .state
            .combat
            .as_ref()
            .map(|combat| combat.entities.values().cloned().collect())
            .unwrap_or_default();
        let data = SaveData {
            metadata: SaveMetadata {
                save_format_version: SAVE_FORMAT_VERSION,
                ruleset_version: self.state.ruleset_version.clone(),
                required_mods: self
                    .state
                    .loaded_mods
                    .iter()
                    .map(|id| RequiredMod {
                        id: id.clone(),
                        version: self
                            .state
                            .loaded_mod_versions
                            .get(id)
                            .cloned()
                            .unwrap_or_else(|| "unknown".into()),
                    })
                    .collect(),
                gamemode_id: self.gamemode_id.clone(),
                player_entity_ids: vec!["base:player".into()],
                rng_state: self.state.rng.state(),
            },
            ruleset: json!({
                "ruleset_version": self.state.ruleset_version,
                "attribute_rules": self.state.attribute_rules,
            }),
            entities,
            combat: self.state.combat.clone(),
            opaque_mod_data: BTreeMap::new(),
            migration_warnings: Vec::new(),
        };
        let path = paths.saves.join(format!("{name}.zip"));
        self.set_diagnostic_context(DiagnosticContext {
            phase: Some("save_write".into()),
            save: Some(name.into()),
            file: Some(path.display().to_string()),
            ..DiagnosticContext::default()
        });
        match write_save(&path, &data) {
            Ok(()) => {
                self.dirty = false;
                format!("Saved {}.", path.display())
            }
            Err(error) => format!("Failed to save {}: {error}", path.display()),
        }
    }

    fn set_log_mode(&mut self, arguments: &[&str]) -> String {
        match arguments.first().copied() {
            Some("normal") => {
                self.verbose = false;
                "Combat output set to normal.".into()
            }
            Some("verbose") => {
                self.verbose = true;
                "Combat output set to verbose.".into()
            }
            _ => "Usage: log normal|verbose".into(),
        }
    }

    fn flee(&mut self) -> String {
        let Some(session) = self.state.combat.as_mut() else {
            return "No combat is active.".into();
        };
        session.flee();
        let explanation = crate::combat::ExplainLast {
            ruleset_version: self.state.ruleset_version.clone(),
            rng_seed: self.state.seed,
            rng_state_before: self.state.rng.state(),
            rng_state_after: self.state.rng.state(),
            actor_id: "base:player".into(),
            action_id: "base:flee".into(),
            target_ids: Vec::new(),
            declared_round: session.round,
            resolved: true,
            rejection_reason: None,
            payment_ap: 0,
            payment_mana: 0,
            eligible_round: session.round,
            queue_position: None,
            action_stages: Vec::new(),
            trigger_order: Vec::new(),
            attack: None,
            damage: None,
            events: Vec::new(),
        };
        session.last_explanation = Some(explanation.clone());
        self.state.explanations.replace(explanation);
        self.mode = AppMode::Ready;
        self.dirty = true;
        "Combat ended: Fled. No combat rewards were granted.".into()
    }

    fn request_quit(&mut self) -> String {
        if self.dirty {
            self.pending_quit = true;
            "Unsaved state exists. Type 'save', 'confirm quit', or 'cancel'.".into()
        } else {
            self.should_exit = true;
            "Goodbye.".into()
        }
    }

    fn dispatch_lua_events(
        &mut self,
        session: &mut crate::session::CombatSession,
        mut cursor: usize,
    ) {
        loop {
            if let Err(error) = process_queued_events(
                session,
                &self.state.status_registry,
                &self.state.attribute_rules,
            ) {
                log_extension_error(
                    &self.state,
                    &format!("Lua event processing failed: {error}"),
                );
                return;
            }
            let events = session.processed_events[cursor..].to_vec();
            if events.is_empty() {
                break;
            }
            cursor = session.processed_events.len();
            let snapshot = LuaWorldSnapshot {
                entities: session
                    .entities
                    .iter()
                    .filter_map(|(id, entity)| {
                        serde_json::to_value(entity)
                            .ok()
                            .map(|value| (id.clone(), value))
                    })
                    .collect(),
            };
            for runtime in &self.lua_runtimes {
                runtime.set_world_snapshot(snapshot.clone());
                for processed in &events {
                    let payload =
                        serde_json::to_value(&processed.event).unwrap_or(serde_json::Value::Null);
                    let event_name = public_event_name(&processed.event);
                    if let Err(error) = runtime.dispatch_event(event_name, &payload) {
                        log_extension_error(
                            &self.state,
                            &format!(
                                "Lua callback failed: mod={}, event={}, listener={}, reason={}",
                                error.mod_id, error.event, error.listener_id, error.reason
                            ),
                        );
                    }
                    session
                        .commands
                        .extend(runtime.drain_commands().into_iter().map(|command| {
                            crate::session::QueuedEngineCommand {
                                depth: processed.depth.saturating_add(1),
                                command,
                            }
                        }));
                }
            }
        }
    }

    fn set_diagnostic_context(&self, context: DiagnosticContext) {
        if let Some(logger) = &self.state.logger {
            let _ = logger.set_context(context);
        }
    }
}

fn load_mod_extensions(state: &mut GameState) -> Vec<ModRuntime> {
    let mut runtimes = Vec::new();
    let roots = state.mod_roots.clone();
    for (mod_id, root) in roots {
        if let Some(logger) = &state.logger {
            let _ = logger.set_context(DiagnosticContext {
                phase: Some("mod_load".into()),
                mod_id: Some(mod_id.clone()),
                file: Some(root.display().to_string()),
                ..DiagnosticContext::default()
            });
        }
        let mut action_registry = state.action_registry.clone();
        let mut status_registry = state.status_registry.clone();
        let action_path = root.join("content").join("actions.json");
        if action_path.is_file() {
            match crate::actions::ActionRegistry::load(&action_path) {
                Ok(registry) => {
                    let definitions = registry
                        .iter()
                        .map(|(_, definition)| definition.clone())
                        .collect::<Vec<_>>();
                    if let Err(error) = definitions
                        .into_iter()
                        .try_for_each(|definition| action_registry.register(definition))
                    {
                        log_extension_error(
                            state,
                            &format!("Action content from {mod_id} failed: {error}"),
                        );
                        continue;
                    }
                }
                Err(error) => {
                    log_extension_error(
                        state,
                        &format!("Action content {} failed: {error}", action_path.display()),
                    );
                    continue;
                }
            }
        }
        let status_path = root.join("content").join("statuses.json");
        if status_path.is_file() {
            match crate::effects::StatusRegistry::load(&status_path) {
                Ok(registry) => {
                    let definitions = registry
                        .iter()
                        .map(|(_, definition)| definition.clone())
                        .collect::<Vec<_>>();
                    if let Err(error) = definitions
                        .into_iter()
                        .try_for_each(|definition| status_registry.register(definition))
                    {
                        log_extension_error(
                            state,
                            &format!("Status content from {mod_id} failed: {error}"),
                        );
                        continue;
                    }
                }
                Err(error) => {
                    log_extension_error(
                        state,
                        &format!("Status content {} failed: {error}", status_path.display()),
                    );
                    continue;
                }
            }
        }
        let runtime = match ModRuntime::new(&mod_id, state.lua_max_instructions) {
            Ok(runtime) => runtime,
            Err(error) => {
                log_extension_error(
                    state,
                    &format!("Failed to initialize Lua for {mod_id}: {error}"),
                );
                continue;
            }
        };
        let scripts = collect_files(&root.join("scripts"), "lua");
        let mut failed = false;
        for script in scripts {
            let source = match fs::read_to_string(&script) {
                Ok(source) => source,
                Err(error) => {
                    log_extension_error(
                        state,
                        &format!(
                            "Failed to read Lua script {} in {mod_id}: {error}",
                            script.display()
                        ),
                    );
                    failed = true;
                    break;
                }
            };
            if let Err(error) = runtime.load_script(&script.display().to_string(), &source) {
                log_extension_error(
                    state,
                    &format!(
                        "Lua script {} in {mod_id} failed: {error}",
                        script.display()
                    ),
                );
                failed = true;
                break;
            }
        }
        if failed {
            continue;
        }
        runtime.finish_loading();
        for value in runtime.take_action_registrations() {
            match serde_json::from_value::<ActionDefinition>(value)
                .map_err(|error| error.to_string())
                .and_then(|definition| {
                    action_registry
                        .register(definition)
                        .map_err(|error| error.to_string())
                }) {
                Ok(()) => {}
                Err(error) => log_extension_error(
                    state,
                    &format!("Action registration from {mod_id} failed: {error}"),
                ),
            }
        }
        for value in runtime.take_status_registrations() {
            match serde_json::from_value(value)
                .map_err(|error| error.to_string())
                .and_then(|definition| {
                    status_registry
                        .register(definition)
                        .map_err(|error| error.to_string())
                }) {
                Ok(()) => {}
                Err(error) => log_extension_error(
                    state,
                    &format!("Status registration from {mod_id} failed: {error}"),
                ),
            }
        }
        if let Err(error) = action_registry.validate(&status_registry) {
            log_extension_error(
                state,
                &format!("Reference validation for {mod_id} failed: {error}"),
            );
            continue;
        }
        state.action_registry = action_registry;
        state.status_registry = status_registry;
        runtime.set_world_snapshot(LuaWorldSnapshot::default());
        runtimes.push(runtime);
    }
    runtimes
}

fn collect_files(root: &Path, extension: &str) -> Vec<std::path::PathBuf> {
    let mut files = fs::read_dir(root)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some(extension))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn log_extension_error(state: &GameState, message: &str) {
    if let Some(logger) = &state.logger {
        let _ = logger.error(message);
    }
}

fn public_event_name(event: &crate::events::CombatEvent) -> &'static str {
    use crate::events::CombatEvent;
    match event {
        CombatEvent::TurnStarted { .. } => "TurnStarted",
        CombatEvent::TurnEnded { .. } => "TurnEnded",
        CombatEvent::OnEvade { .. } => "OnEvade",
        CombatEvent::OnParry { .. } => "OnParry",
        CombatEvent::OnBlock { .. } => "OnBlock",
        CombatEvent::OnAttackReceived { .. } => "OnAttackReceived",
        CombatEvent::OnDamageReceived { .. } => "OnDamageReceived",
        CombatEvent::OnShieldDamage { .. } => "OnShieldDamage",
        CombatEvent::OnTenacityDamage { .. } => "OnTenacityDamage",
        CombatEvent::OnHpDamage { .. } => "OnHPDamage",
        CombatEvent::OnHpBelow { .. } => "OnHPBelow",
        CombatEvent::OnKill { .. } => "OnKill",
        CombatEvent::StatusApplied { .. } => "StatusApplied",
        CombatEvent::StatusRemoved { .. } => "StatusRemoved",
    }
}
