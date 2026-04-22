use std::collections::BTreeSet;

pub struct CommandContribution {
    pub command_id: String,
}

impl CommandContribution {
    pub fn new(command_id: impl Into<String>) -> Self {
        Self {
            command_id: command_id.into(),
        }
    }
}

pub trait FeatureModule {
    fn module_id(&self) -> &str;
    fn command_contributions(&self) -> &[CommandContribution];
}

pub struct SimpleModule {
    module_id: String,
    commands: Vec<CommandContribution>,
}

impl SimpleModule {
    pub fn with_command(module_id: impl Into<String>, command: CommandContribution) -> Self {
        Self {
            module_id: module_id.into(),
            commands: vec![command],
        }
    }
}

impl FeatureModule for SimpleModule {
    fn module_id(&self) -> &str {
        &self.module_id
    }

    fn command_contributions(&self) -> &[CommandContribution] {
        &self.commands
    }
}

#[derive(Default)]
pub struct ModuleRuntime {
    module_ids: BTreeSet<String>,
    command_ids: BTreeSet<String>,
}

impl ModuleRuntime {
    pub fn register(&mut self, module: Box<dyn FeatureModule>) -> Result<(), String> {
        let module_id = module.module_id().to_owned();
        if self.module_ids.contains(&module_id) {
            return Err(format!("duplicate module id: {module_id}"));
        }

        let snapshot_command_ids: Vec<String> = module
            .command_contributions()
            .iter()
            .map(|contribution| contribution.command_id.clone())
            .collect();

        let mut incoming_command_ids = BTreeSet::new();
        for command_id in &snapshot_command_ids {
            if !incoming_command_ids.insert(command_id.clone()) {
                return Err(format!("duplicate command contribution id: {command_id}"));
            }
            if self.command_ids.contains(command_id) {
                return Err(format!("duplicate command contribution id: {command_id}"));
            }
        }

        self.module_ids.insert(module_id);
        for command_id in snapshot_command_ids {
            self.command_ids.insert(command_id);
        }

        Ok(())
    }
}
