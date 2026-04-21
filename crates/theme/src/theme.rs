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
        tokens.insert(
            "status_bar.background".to_string(),
            "#1f2937".to_string(),
        );

        Self {
            name: "Default".to_string(),
            tokens,
        }
    }
}
