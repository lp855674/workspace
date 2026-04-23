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
        tokens.insert("app.background".to_string(), "#111111".to_string());
        tokens.insert("sidebar.background".to_string(), "#1a1a1a".to_string());
        tokens.insert("status_bar.background".to_string(), "#222222".to_string());

        Self {
            name: "Default".to_string(),
            tokens,
        }
    }
}
