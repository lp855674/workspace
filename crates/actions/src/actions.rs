use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum ActionEnvelope {
    #[serde(rename = "panel")]
    Panel(PanelActionEnvelope),
    #[serde(rename = "noop")]
    Noop,
}

impl ActionEnvelope {
    pub fn panel(action: PanelAction) -> Self {
        Self::Panel(PanelActionEnvelope::from(action))
    }

    pub fn noop() -> Self {
        Self::Noop
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Panel(_) => "panel",
            Self::Noop => "noop",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PanelAction {
    Open { panel_type_id: String },
    Toggle { panel_type_id: String },
    Focus { panel_type_id: String },
    Close { panel_type_id: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PanelActionEnvelope {
    pub action: String,
    pub panel_type_id: String,
}

impl From<PanelAction> for PanelActionEnvelope {
    fn from(value: PanelAction) -> Self {
        match value {
            PanelAction::Open { panel_type_id } => Self {
                action: "open".to_owned(),
                panel_type_id,
            },
            PanelAction::Toggle { panel_type_id } => Self {
                action: "toggle".to_owned(),
                panel_type_id,
            },
            PanelAction::Focus { panel_type_id } => Self {
                action: "focus".to_owned(),
                panel_type_id,
            },
            PanelAction::Close { panel_type_id } => Self {
                action: "close".to_owned(),
                panel_type_id,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionDescriptor {
    id: String,
}

impl ActionDescriptor {
    pub fn try_new(id: impl Into<String>) -> Result<Self, ActionDescriptorError> {
        let id = id.into();
        let trimmed = id.trim();
        if trimmed.is_empty() {
            return Err(ActionDescriptorError::EmptyId);
        }

        Ok(Self {
            id: trimmed.to_owned(),
        })
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionDescriptorError {
    EmptyId,
}

impl fmt::Display for ActionDescriptorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => formatter.write_str("action id cannot be empty"),
        }
    }
}

impl Error for ActionDescriptorError {}
