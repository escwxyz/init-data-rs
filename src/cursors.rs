use nvim_utils::prelude::*;
use once_cell::sync::OnceCell;
use std::sync::RwLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct CursorPosition(u64, u64);

static CURSOR_POSITIONS: OnceCell<RwLock<Vec<CursorPosition>>> = OnceCell::new();

// public functions
// TODO not working properly
pub fn toggle_cursor_position(lua: &Lua, _: ()) -> LuaResult<()> {
    let mut write = global().write().unwrap();

    let position = get_current_cursor(lua)?;

    if let Some(index) = write.iter().position(|p| p == &position) {
        write.remove(index);
    } else {
        drop(write); // Release write lock
        let mut write = global().write().unwrap(); // Re-acquire write lock
        write.push(position);
    }

    // TODO for debug only

    log::info(
        lua,
        format!("Current cursors: {:?}", get_cursors()?).as_str(),
    )?;

    Ok(())
}

pub fn move_cursor_down(lua: &Lua, _: ()) -> LuaResult<()> {
    let CursorPosition(line, col) = get_current_cursor(lua)?;

    let line_count = get_line_count(lua)?;

    log::info(lua, format!("Line count is {}", line_count).as_str())?;

    let next_line = if line_count == line { line } else { line + 1 };

    vim::api::nvim_win_set_cursor(lua, 0, (next_line, col))?;

    log::info(lua, format!("Move to line: {}", next_line).as_str())?;

    add_cursor(next_line, col)?;

    // TODO for debug only

    log::info(
        lua,
        format!("Current cursors: {:?}", get_cursors()?).as_str(),
    )?;

    Ok(())
}

pub fn move_cursor_up(lua: &Lua, _: ()) -> LuaResult<()> {
    let CursorPosition(line, col) = get_current_cursor(lua)?;

    let next_line = if line == 1 { 1 } else { line - 1 };

    vim::api::nvim_win_set_cursor(lua, 0, (next_line, col))?;

    remove_cursor(line, col)?;

    Ok(())
}

// internal functions

#[allow(dead_code)]
fn global() -> &'static RwLock<Vec<CursorPosition>> {
    CURSOR_POSITIONS.get_or_init(|| RwLock::new(Vec::new()))
}

fn add_cursor(line: u64, col: u64) -> LuaResult<()> {
    let mut write = global().write().unwrap();

    let position = CursorPosition(line, col);

    if !write.contains(&position) {
        write.push(CursorPosition(line, col));
    }

    Ok(())
}

fn remove_cursor(line: u64, col: u64) -> LuaResult<()> {
    let mut write = global().write().unwrap();

    let position = CursorPosition(line, col);

    if let Some(idx) = write.iter().position(|p| p == &position) {
        write.remove(idx);
    }

    Ok(())
}

fn get_cursors() -> LuaResult<Vec<CursorPosition>> {
    let read = global().read().unwrap();

    Ok(read.clone())
}

#[allow(dead_code)]
fn reset_cursors() -> LuaResult<()> {
    let mut write = global().write().unwrap();

    write.clear();

    Ok(())
}

fn get_current_cursor(lua: &Lua) -> LuaResult<CursorPosition> {
    let pos = vim::api::nvim_win_get_cursor(lua, 0)?;
    let line = pos.get::<u64, u64>(1)?;
    let col = pos.get::<u64, u64>(2)?;

    Ok(CursorPosition(line, col))
}

fn get_line_count(lua: &Lua) -> LuaResult<u64> {
    vim::api::get(lua)?.call_function("nvim_buf_line_count", 0)
}
