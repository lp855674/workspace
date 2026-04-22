use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandDescriptor {
    pub id: String,
    pub title: String,
}

impl CommandDescriptor {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CommandRegistry {
    commands: BTreeMap<String, CommandDescriptor>,
}

impl CommandRegistry {
    pub fn register(&mut self, command: CommandDescriptor) -> Result<(), String> {
        if self.commands.contains_key(&command.id) {
            return Err(format!("duplicate command id: {}", command.id));
        }

        self.commands.insert(command.id.clone(), command);
        Ok(())
    }
}
