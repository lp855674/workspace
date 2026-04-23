use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use actions::ActionEnvelope;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CommandId(String);

impl CommandId {
    pub fn try_new(value: impl Into<String>) -> Result<Self, CommandDescriptorError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(CommandDescriptorError::EmptyId);
        }

        if trimmed.chars().any(char::is_whitespace) {
            return Err(CommandDescriptorError::IdContainsWhitespace);
        }

        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommandId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandTitle(String);

impl CommandTitle {
    pub fn try_new(value: impl Into<String>) -> Result<Self, CommandDescriptorError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(CommandDescriptorError::EmptyTitle);
        }

        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CommandTitle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandDescriptorError {
    EmptyId,
    IdContainsWhitespace,
    EmptyTitle,
}

impl fmt::Display for CommandDescriptorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => formatter.write_str("command id cannot be empty"),
            Self::IdContainsWhitespace => {
                formatter.write_str("command id cannot contain whitespace")
            }
            Self::EmptyTitle => formatter.write_str("command title cannot be empty"),
        }
    }
}

impl Error for CommandDescriptorError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandDescriptor {
    id: CommandId,
    title: CommandTitle,
    action: ActionEnvelope,
}

impl CommandDescriptor {
    pub fn try_new(
        id: impl Into<String>,
        title: impl Into<String>,
    ) -> Result<Self, CommandDescriptorError> {
        Self::try_new_with_action(id, title, ActionEnvelope::noop())
    }

    pub fn try_new_with_action(
        id: impl Into<String>,
        title: impl Into<String>,
        action: ActionEnvelope,
    ) -> Result<Self, CommandDescriptorError> {
        Ok(Self {
            id: CommandId::try_new(id)?,
            title: CommandTitle::try_new(title)?,
            action,
        })
    }

    pub fn id(&self) -> &CommandId {
        &self.id
    }

    pub fn title(&self) -> &CommandTitle {
        &self.title
    }

    pub fn action(&self) -> &ActionEnvelope {
        &self.action
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegisterCommandError {
    DuplicateCommandId(CommandId),
}

impl fmt::Display for RegisterCommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCommandId(command_id) => {
                write!(formatter, "duplicate command id: {command_id}")
            }
        }
    }
}

impl Error for RegisterCommandError {}

#[derive(Debug, Default)]
pub struct CommandRegistry {
    commands: BTreeMap<CommandId, CommandDescriptor>,
}

impl CommandRegistry {
    pub fn register(&mut self, command: CommandDescriptor) -> Result<(), RegisterCommandError> {
        if self.commands.contains_key(command.id()) {
            return Err(RegisterCommandError::DuplicateCommandId(command.id.clone()));
        }

        self.commands.insert(command.id.clone(), command);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn commands(&self) -> impl Iterator<Item = &CommandDescriptor> {
        self.commands.values()
    }
}
