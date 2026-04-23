use keymap::KeyBinding;
use theme::ThemeDefinition;

pub const DEFAULT_THEME_JSON: &str = include_str!("../../../assets/themes/default.json");
pub const DEFAULT_WINDOWS_KEYMAP_JSON: &str =
    include_str!("../../../assets/keymaps/default-windows.json");

pub fn load_default_theme() -> Result<ThemeDefinition, serde_json::Error> {
    serde_json::from_str(DEFAULT_THEME_JSON)
}

pub fn load_windows_keymap() -> Result<Vec<KeyBinding>, serde_json::Error> {
    serde_json::from_str(DEFAULT_WINDOWS_KEYMAP_JSON)
}
