use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use commands::{CommandDescriptorError, CommandId};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModuleId(String);

impl ModuleId {
    pub fn try_new(value: impl Into<String>) -> Result<Self, ModuleIdError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(ModuleIdError::EmptyId);
        }

        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ModuleId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleIdError {
    EmptyId,
}

impl fmt::Display for ModuleIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => formatter.write_str("module id cannot be empty"),
        }
    }
}

impl Error for ModuleIdError {}

pub struct CommandContribution {
    command_id: CommandId,
}

impl CommandContribution {
    pub fn new(command_id: CommandId) -> Self {
        Self { command_id }
    }

    pub fn try_new(command_id: impl Into<String>) -> Result<Self, CommandDescriptorError> {
        Ok(Self {
            command_id: CommandId::try_new(command_id)?,
        })
    }

    pub fn command_id(&self) -> &CommandId {
        &self.command_id
    }
}

pub trait FeatureModule {
    fn module_id(&self) -> &str;
    fn command_contributions(&self) -> &[CommandContribution];
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegisteredModule {
    module_id: ModuleId,
    command_ids: Vec<CommandId>,
}

impl RegisteredModule {
    fn try_snapshot(module: &dyn FeatureModule) -> Result<(Self, Vec<CommandId>), RegisterModuleError> {
        let module_id =
            ModuleId::try_new(module.module_id()).map_err(RegisterModuleError::InvalidModuleId)?;
        let command_ids: Vec<CommandId> = module
            .command_contributions()
            .iter()
            .map(|contribution| contribution.command_id().clone())
            .collect();

        Ok((
            Self {
                module_id,
                command_ids: command_ids.clone(),
            },
            command_ids,
        ))
    }

    pub fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn command_ids(&self) -> &[CommandId] {
        &self.command_ids
    }
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RegisterModuleError {
    InvalidModuleId(ModuleIdError),
    DuplicateModuleId(ModuleId),
    DuplicateCommandId(CommandId),
}

impl fmt::Display for RegisterModuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidModuleId(error) => write!(formatter, "invalid module id: {error}"),
            Self::DuplicateModuleId(module_id) => write!(formatter, "duplicate module id: {module_id}"),
            Self::DuplicateCommandId(command_id) => {
                write!(formatter, "duplicate command contribution id: {command_id}")
            }
        }
    }
}

impl Error for RegisterModuleError {}

#[derive(Default)]
pub struct ModuleRuntime {
    module_ids: BTreeSet<ModuleId>,
    command_ids: BTreeSet<CommandId>,
    modules: Vec<RegisteredModule>,
    live_modules: Vec<Box<dyn FeatureModule>>,
}

impl ModuleRuntime {
    pub fn register(&mut self, module: Box<dyn FeatureModule>) -> Result<(), RegisterModuleError> {
        let (retained_module, snapshot_command_ids) = RegisteredModule::try_snapshot(module.as_ref())?;
        let module_id = retained_module.module_id().clone();
        if self.module_ids.contains(&module_id) {
            return Err(RegisterModuleError::DuplicateModuleId(module_id));
        }

        let mut incoming_command_ids = BTreeSet::new();
        for command_id in &snapshot_command_ids {
            if !incoming_command_ids.insert(command_id.clone()) {
                return Err(RegisterModuleError::DuplicateCommandId(command_id.clone()));
            }
            if self.command_ids.contains(command_id) {
                return Err(RegisterModuleError::DuplicateCommandId(command_id.clone()));
            }
        }

        self.module_ids.insert(module_id);
        for command_id in snapshot_command_ids {
            self.command_ids.insert(command_id);
        }
        self.modules.push(retained_module);
        self.live_modules.push(module);

        Ok(())
    }

    pub fn retained_module_count(&self) -> usize {
        self.modules.len()
    }

    pub fn retained_modules(&self) -> &[RegisteredModule] {
        &self.modules
    }
}
