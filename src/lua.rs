use mlua::{Lua, LuaOptions, StdLib};
pub const MOD_API_VERSION: u32 = 1;

/// Creates a sandboxed Lua 5.4 runtime. IO, OS, package loading, native modules,
/// and debug access are deliberately absent.
pub fn create_runtime() -> mlua::Result<Lua> {
    let libraries =
        StdLib::COROUTINE | StdLib::TABLE | StdLib::STRING | StdLib::UTF8 | StdLib::MATH;
    let lua = Lua::new_with(libraries, LuaOptions::default())?;
    lua.globals().set("mod_api_version", MOD_API_VERSION)?;
    let game = lua.create_table()?;
    game.set("mod_api_version", MOD_API_VERSION)?;
    lua.globals().set("game", game)?;
    Ok(lua)
}
