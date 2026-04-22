use commands::{CommandDescriptor, CommandId, CommandTitle};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MenuItem {
    command: CommandDescriptor,
}

impl MenuItem {
    pub fn from_command(command: CommandDescriptor) -> Self {
        Self { command }
    }

    pub fn command(&self) -> &CommandDescriptor {
        &self.command
    }

    pub fn command_id(&self) -> &CommandId {
        self.command.id()
    }

    pub fn title(&self) -> &CommandTitle {
        self.command.title()
    }
}
