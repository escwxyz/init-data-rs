use once_cell::sync::OnceCell;

pub enum Keymap {
    AddCursor(String),
    RemoveCursor(String),
}

pub struct Config {
    keymaps: Vec<Keymap>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keymaps: vec![
                Keymap::AddCursor("<C-n>".to_string()),
                Keymap::RemoveCursor("<C-p>".to_string()),
            ],
        }
    }
}

pub static CONFIG: OnceCell<Config> = OnceCell::new();
