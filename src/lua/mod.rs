use crate::{combat::DamageType, effects::ActiveStatus, numeric::GameInt, session::EngineCommand};
use mlua::{Error, HookTriggers, Lua, LuaOptions, LuaSerdeExt, StdLib, Table, Value, VmState};
use serde_json::Value as JsonValue;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        atomic::{AtomicBool, AtomicI64, Ordering},
        Arc, Mutex,
    },
};

pub const MOD_API_VERSION: u32 = 1;
pub const DEFAULT_MAX_INSTRUCTIONS: u64 = 100_000;

pub fn create_runtime() -> mlua::Result<Lua> {
    create_runtime_with_limit(DEFAULT_MAX_INSTRUCTIONS)
}

fn create_runtime_with_limit(max_instructions: u64) -> mlua::Result<Lua> {
    let libraries =
        StdLib::COROUTINE | StdLib::TABLE | StdLib::STRING | StdLib::UTF8 | StdLib::MATH;
    let lua = Lua::new_with(libraries, LuaOptions::default())?;
    lua.globals().set("mod_api_version", MOD_API_VERSION)?;
    let game = lua.create_table()?;
    game.set("mod_api_version", MOD_API_VERSION)?;
    lua.globals().set("game", game)?;
    let remaining = Arc::new(AtomicI64::new(max_instructions.min(i64::MAX as u64) as i64));
    lua.set_app_data(remaining.clone());
    lua.set_hook(
        HookTriggers::new().every_nth_instruction(1_000),
        move |_lua, _debug| {
            if remaining.fetch_sub(1_000, Ordering::Relaxed) <= 1_000 {
                Err(Error::runtime("Lua callback instruction limit exceeded"))
            } else {
                Ok(VmState::Continue)
            }
        },
    );
    Ok(lua)
}

#[derive(Clone, Debug, Default)]
pub struct LuaWorldSnapshot {
    pub entities: BTreeMap<String, JsonValue>,
}

#[derive(Clone, Debug)]
pub struct LuaCallbackError {
    pub mod_id: String,
    pub event: String,
    pub listener_id: String,
    pub reason: String,
}

pub struct ModRuntime {
    lua: Lua,
    mod_id: String,
    max_instructions: u64,
    commands: Arc<Mutex<VecDeque<EngineCommand>>>,
    action_registrations: Arc<Mutex<Vec<JsonValue>>>,
    status_registrations: Arc<Mutex<Vec<JsonValue>>>,
    world: Arc<Mutex<LuaWorldSnapshot>>,
    loading: Arc<AtomicBool>,
}

impl ModRuntime {
    pub fn new(mod_id: impl Into<String>, max_instructions: u64) -> mlua::Result<Self> {
        let mod_id = mod_id.into();
        let lua = create_runtime_with_limit(max_instructions)?;
        let commands = Arc::new(Mutex::new(VecDeque::new()));
        let action_registrations = Arc::new(Mutex::new(Vec::new()));
        let status_registrations = Arc::new(Mutex::new(Vec::new()));
        let world = Arc::new(Mutex::new(LuaWorldSnapshot::default()));
        let loading = Arc::new(AtomicBool::new(true));
        install_api(
            &lua,
            &mod_id,
            commands.clone(),
            action_registrations.clone(),
            status_registrations.clone(),
            world.clone(),
            loading.clone(),
        )?;
        Ok(Self {
            lua,
            mod_id,
            max_instructions,
            commands,
            action_registrations,
            status_registrations,
            world,
            loading,
        })
    }

    pub fn load_script(&self, name: &str, source: &str) -> mlua::Result<()> {
        self.reset_instruction_budget();
        self.lua.load(source).set_name(name).exec()
    }

    pub fn set_world_snapshot(&self, snapshot: LuaWorldSnapshot) {
        if let Ok(mut world) = self.world.lock() {
            *world = snapshot;
        }
    }

    pub fn finish_loading(&self) {
        self.loading.store(false, Ordering::Relaxed);
    }

    pub fn dispatch_event(
        &self,
        event: &str,
        payload: &JsonValue,
    ) -> Result<Vec<String>, LuaCallbackError> {
        let callbacks: Table = self
            .lua
            .named_registry_value("ams.callbacks")
            .map_err(|error| self.callback_error(event, "registry", error))?;
        let Some(event_callbacks) = callbacks
            .get::<Option<Table>>(event)
            .map_err(|error| self.callback_error(event, "registry", error))?
        else {
            return Ok(Vec::new());
        };
        let mut listeners = event_callbacks
            .clone()
            .pairs::<String, mlua::Function>()
            .collect::<mlua::Result<Vec<_>>>()
            .map_err(|error| self.callback_error(event, "registry", error))?;
        listeners.sort_by(|left, right| left.0.cmp(&right.0));
        let mut called = Vec::new();
        for (listener_id, callback) in listeners {
            self.reset_instruction_budget();
            let value = self
                .lua
                .to_value(payload)
                .map_err(|error| self.callback_error(event, &listener_id, error))?;
            if let Err(error) = callback.call::<()>(value) {
                event_callbacks
                    .set(listener_id.as_str(), Value::Nil)
                    .map_err(|set_error| self.callback_error(event, &listener_id, set_error))?;
                return Err(self.callback_error(event, &listener_id, error));
            }
            called.push(listener_id);
        }
        Ok(called)
    }

    pub fn drain_commands(&self) -> Vec<EngineCommand> {
        self.commands
            .lock()
            .map(|mut commands| commands.drain(..).collect())
            .unwrap_or_default()
    }

    pub fn take_action_registrations(&self) -> Vec<JsonValue> {
        self.action_registrations
            .lock()
            .map(|mut values| std::mem::take(&mut *values))
            .unwrap_or_default()
    }

    pub fn take_status_registrations(&self) -> Vec<JsonValue> {
        self.status_registrations
            .lock()
            .map(|mut values| std::mem::take(&mut *values))
            .unwrap_or_default()
    }

    fn reset_instruction_budget(&self) {
        if let Some(remaining) = self.lua.app_data_ref::<Arc<AtomicI64>>() {
            remaining.store(
                self.max_instructions.min(i64::MAX as u64) as i64,
                Ordering::Relaxed,
            );
        }
    }

    fn callback_error(&self, event: &str, listener_id: &str, error: Error) -> LuaCallbackError {
        LuaCallbackError {
            mod_id: self.mod_id.clone(),
            event: event.into(),
            listener_id: listener_id.into(),
            reason: error.to_string(),
        }
    }
}

fn install_api(
    lua: &Lua,
    mod_id: &str,
    commands: Arc<Mutex<VecDeque<EngineCommand>>>,
    actions: Arc<Mutex<Vec<JsonValue>>>,
    statuses: Arc<Mutex<Vec<JsonValue>>>,
    world: Arc<Mutex<LuaWorldSnapshot>>,
    loading: Arc<AtomicBool>,
) -> mlua::Result<()> {
    let game: Table = lua.globals().get("game")?;
    let callbacks = lua.create_table()?;
    lua.set_named_registry_value("ams.callbacks", callbacks.clone())?;

    let namespace = mod_id.split(':').next().unwrap_or(mod_id).to_owned();
    let on_callbacks = callbacks.clone();
    game.set(
        "on",
        lua.create_function(
            move |lua, (event, listener_id, callback): (String, String, mlua::Function)| {
                if !listener_id.starts_with(&format!("{namespace}:")) {
                    return Err(Error::runtime("listener_id must use the mod namespace"));
                }
                let table = match on_callbacks.get::<Option<Table>>(event.as_str())? {
                    Some(table) => table,
                    None => {
                        let table = lua.create_table()?;
                        on_callbacks.set(event.as_str(), table.clone())?;
                        table
                    }
                };
                if !table.get::<Value>(listener_id.as_str())?.is_nil() {
                    return Err(Error::runtime("duplicate listener_id"));
                }
                table.set(listener_id, callback)
            },
        )?,
    )?;

    let action_values = actions.clone();
    let action_loading = loading.clone();
    game.set(
        "register_action",
        lua.create_function(move |lua, value: Value| {
            if !action_loading.load(Ordering::Relaxed) {
                return Err(Error::runtime("register_action is load-time only"));
            }
            let json: JsonValue = lua.from_value(value)?;
            action_values
                .lock()
                .map_err(|_| Error::runtime("action registration lock poisoned"))?
                .push(json);
            Ok(())
        })?,
    )?;
    let status_values = statuses.clone();
    let status_loading = loading;
    game.set(
        "register_status",
        lua.create_function(move |lua, value: Value| {
            if !status_loading.load(Ordering::Relaxed) {
                return Err(Error::runtime("register_status is load-time only"));
            }
            let json: JsonValue = lua.from_value(value)?;
            status_values
                .lock()
                .map_err(|_| Error::runtime("status registration lock poisoned"))?
                .push(json);
            Ok(())
        })?,
    )?;

    let query_world = world.clone();
    game.set(
        "get_entity",
        lua.create_function(move |lua, id: String| {
            let value = query_world
                .lock()
                .map_err(|_| Error::runtime("world snapshot lock poisoned"))?
                .entities
                .get(&id)
                .cloned();
            value.map(|value| lua.to_value(&value)).transpose()
        })?,
    )?;
    let attribute_world = world.clone();
    game.set(
        "get_attribute",
        lua.create_function(move |_, (id, attribute): (String, String)| {
            Ok(attribute_world
                .lock()
                .map_err(|_| Error::runtime("world snapshot lock poisoned"))?
                .entities
                .get(&id)
                .and_then(|entity| entity.get("attributes"))
                .and_then(|attributes| attributes.get("values"))
                .and_then(|values| values.get(&attribute))
                .and_then(JsonValue::as_i64))
        })?,
    )?;
    let resource_world = world.clone();
    game.set(
        "get_resource",
        lua.create_function(move |lua, (id, resource): (String, String)| {
            let world = resource_world
                .lock()
                .map_err(|_| Error::runtime("world snapshot lock poisoned"))?;
            let value = world
                .entities
                .get(&id)
                .and_then(|entity| entity.get(&resource));
            value.cloned().map(|value| lua.to_value(&value)).transpose()
        })?,
    )?;
    let status_world = world.clone();
    game.set(
        "has_status",
        lua.create_function(move |_, (id, status_id): (String, String)| {
            Ok(status_world
                .lock()
                .map_err(|_| Error::runtime("world snapshot lock poisoned"))?
                .entities
                .get(&id)
                .and_then(|entity| entity.get("statuses"))
                .and_then(|statuses| statuses.get("active"))
                .and_then(|active| active.get(&status_id))
                .is_some())
        })?,
    )?;

    let damage_commands = commands.clone();
    game.set(
        "damage",
        lua.create_function(
            move |_, (source_id, target_id, amount, kind): (String, String, GameInt, String)| {
                let damage_type = match kind.as_str() {
                    "physical" => DamageType::Physical,
                    "magical" => DamageType::Magical,
                    "true" => DamageType::True,
                    _ => return Err(Error::runtime("unknown damage type")),
                };
                damage_commands
                    .lock()
                    .map_err(|_| Error::runtime("command queue lock poisoned"))?
                    .push_back(EngineCommand::Damage {
                        source_id,
                        target_id,
                        amount,
                        damage_type,
                    });
                Ok(())
            },
        )?,
    )?;
    let heal_commands = commands.clone();
    game.set(
        "heal",
        lua.create_function(move |_, (target_id, amount): (String, GameInt)| {
            heal_commands
                .lock()
                .map_err(|_| Error::runtime("command queue lock poisoned"))?
                .push_back(EngineCommand::Heal { target_id, amount });
            Ok(())
        })?,
    )?;
    let resource_commands = commands.clone();
    game.set(
        "change_resource",
        lua.create_function(
            move |_, (target_id, resource, amount): (String, String, GameInt)| {
                if resource != "mana" {
                    return Err(Error::runtime(
                        "API v1 currently supports the mana resource",
                    ));
                }
                resource_commands
                    .lock()
                    .map_err(|_| Error::runtime("command queue lock poisoned"))?
                    .push_back(EngineCommand::ChangeMana { target_id, amount });
                Ok(())
            },
        )?,
    )?;
    let apply_commands = commands.clone();
    game.set(
        "apply_status",
        lua.create_function(
            move |_,
                  (target_id, effect_id, source_id, potency, counter): (
                String,
                String,
                String,
                GameInt,
                GameInt,
            )| {
                apply_commands
                    .lock()
                    .map_err(|_| Error::runtime("command queue lock poisoned"))?
                    .push_back(EngineCommand::ApplyStatus {
                        target_id,
                        status: ActiveStatus::new(effect_id, source_id, potency, counter),
                    });
                Ok(())
            },
        )?,
    )?;
    game.set(
        "remove_status",
        lua.create_function(move |_, (target_id, effect_id): (String, String)| {
            commands
                .lock()
                .map_err(|_| Error::runtime("command queue lock poisoned"))?
                .push_back(EngineCommand::RemoveStatus {
                    target_id,
                    effect_id,
                });
            Ok(())
        })?,
    )?;
    Ok(())
}
