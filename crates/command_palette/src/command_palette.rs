use commands::CommandDescriptor;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CommandPaletteState {
    is_open: bool,
    query: String,
}

impl CommandPaletteState {
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
        self.query.clear();
    }

    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into().trim().to_lowercase();
    }

    pub fn matching_commands<'a>(
        &self,
        commands: &'a [CommandDescriptor],
    ) -> Vec<&'a CommandDescriptor> {
        if self.query.is_empty() {
            return commands.iter().collect();
        }

        commands
            .iter()
            .filter(|command| {
                command.id().as_str().contains(&self.query)
                    || command
                        .title()
                        .as_str()
                        .to_lowercase()
                        .contains(&self.query)
            })
            .collect()
    }
}
