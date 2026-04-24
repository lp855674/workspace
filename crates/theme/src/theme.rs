use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThemeDefinition {
    pub name: String,
    pub tokens: BTreeMap<String, String>,
}

impl Default for ThemeDefinition {
    fn default() -> Self {
        let mut tokens = BTreeMap::new();
        tokens.insert("app.background".to_string(), "#0f1419".to_string());
        tokens.insert("title_bar.background".to_string(), "#10161d".to_string());
        tokens.insert("title_bar.foreground".to_string(), "#e1e8ef".to_string());
        tokens.insert("title_bar.border".to_string(), "#1f2a33".to_string());
        tokens.insert(
            "window_control.foreground".to_string(),
            "#f2f7fb".to_string(),
        );
        tokens.insert("window_control.hover".to_string(), "#2b3844".to_string());
        tokens.insert(
            "window_control.close_hover".to_string(),
            "#c42b1c".to_string(),
        );
        tokens.insert("sidebar.background".to_string(), "#121921".to_string());
        tokens.insert("sidebar.foreground".to_string(), "#cdd8e2".to_string());
        tokens.insert("sidebar.muted".to_string(), "#8092a2".to_string());
        tokens.insert("dock.background".to_string(), "#0f1419".to_string());
        tokens.insert("dock.header".to_string(), "#10161d".to_string());
        tokens.insert("border".to_string(), "#1c2730".to_string());
        tokens.insert("resize_handle.hover".to_string(), "#4d8fd6".to_string());
        tokens.insert("status_bar.background".to_string(), "#111820".to_string());
        tokens.insert("status_bar.foreground".to_string(), "#8fa2b4".to_string());
        tokens.insert("status_bar.active".to_string(), "#dce7f0".to_string());
        tokens.insert("status_bar.button".to_string(), "#141d26".to_string());
        tokens.insert(
            "status_bar.button_active".to_string(),
            "#1b2834".to_string(),
        );
        tokens.insert("status_bar.button_hover".to_string(), "#24313d".to_string());

        Self {
            name: "Default".to_string(),
            tokens,
        }
    }
}

impl ThemeDefinition {
    pub fn token_hex(&self, token: &str, fallback: u32) -> u32 {
        self.tokens
            .get(token)
            .and_then(|value| parse_hex_color(value))
            .unwrap_or(fallback)
    }
}

fn parse_hex_color(value: &str) -> Option<u32> {
    let value = value.trim().strip_prefix('#').unwrap_or(value.trim());
    if value.len() != 6 {
        return None;
    }
    u32::from_str_radix(value, 16).ok()
}
