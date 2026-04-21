use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBinding {
    pub command_id: String,
    pub keystroke: String,
}

impl KeyBinding {
    pub fn new(command_id: impl Into<String>, keystroke: impl Into<String>) -> Self {
        Self {
            command_id: command_id.into(),
            keystroke: keystroke.into(),
        }
    }
}
