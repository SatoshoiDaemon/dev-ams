use mlua::Lua;
pub const MOD_API_VERSION: u32 = 1;
pub fn create_runtime() -> Lua { Lua::new() }
