use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub restore_session_on_launch: bool,
    pub theme_name: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            restore_session_on_launch: true,
            theme_name: "Default".to_string(),
        }
    }
}
