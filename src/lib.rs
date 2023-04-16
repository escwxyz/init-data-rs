mod config;
mod cursors;
mod utils;

use nvim_utils::prelude::*;

// call lua function
// vim::api::get(lua)?.call_function(lua, key, args)
// https://docs.rs/nvim-utils/latest/nvim_utils/prelude/struct.LuaTable.html#method.call_function

// #[allow(dead_code)]
// fn init(config: Option<Config>) -> LuaResult<()> {
//     // TODO
//     let default_config = CONFIG.get_or_init(|| Config::default());

//     Ok(())
// }

fn test(lua: &Lua, _args: ()) -> LuaResult<()> {
    Ok(())
}

#[mlua::lua_module]
fn cursors(lua: &Lua) -> LuaResult<LuaTable> {
    ModuleBuilder::new(lua)
        .with_fn("test", test)?
        .with_fn("move_cursor_down", cursors::move_cursor_down)?
        .with_fn("move_cursor_up", cursors::move_cursor_up)?
        .with_fn("toggle_cursor_position", cursors::toggle_cursor_position)?
        .build()
}
