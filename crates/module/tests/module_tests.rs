use commands::CommandDescriptorError;
use module::{
    CommandContribution, FeatureModule, ModuleIdError, ModuleRuntime, RegisterModuleError,
    SimpleModule,
};
use std::cell::Cell;

fn command(command_id: &str) -> CommandContribution {
    CommandContribution::try_new(command_id).unwrap()
}

struct TestModule {
    module_id: String,
    commands: Vec<CommandContribution>,
}

impl TestModule {
    fn new(module_id: impl Into<String>, commands: Vec<CommandContribution>) -> Self {
        Self {
            module_id: module_id.into(),
            commands,
        }
    }
}

impl FeatureModule for TestModule {
    fn module_id(&self) -> &str {
        &self.module_id
    }

    fn command_contributions(&self) -> &[CommandContribution] {
        &self.commands
    }
}

struct StatefulModule {
    module_id: String,
    first_snapshot: Vec<CommandContribution>,
    second_snapshot: Vec<CommandContribution>,
    call_count: Cell<u8>,
}

impl StatefulModule {
    fn new(
        module_id: impl Into<String>,
        first_snapshot: Vec<CommandContribution>,
        second_snapshot: Vec<CommandContribution>,
    ) -> Self {
        Self {
            module_id: module_id.into(),
            first_snapshot,
            second_snapshot,
            call_count: Cell::new(0),
        }
    }
}

impl FeatureModule for StatefulModule {
    fn module_id(&self) -> &str {
        &self.module_id
    }

    fn command_contributions(&self) -> &[CommandContribution] {
        let call_count = self.call_count.get();
        self.call_count.set(call_count.saturating_add(1));
        if call_count == 0 {
            &self.first_snapshot
        } else {
            &self.second_snapshot
        }
    }
}

#[test]
fn module_runtime_rejects_duplicate_command_ids_across_modules() {
    let first = SimpleModule::with_command("welcome", command("command.open"));
    let second = SimpleModule::with_command("inspector", command("command.open"));

    let mut runtime = ModuleRuntime::default();
    assert!(runtime.register(Box::new(first)).is_ok());
    let result = runtime.register(Box::new(second));
    assert!(matches!(
        result,
        Err(RegisterModuleError::DuplicateCommandId(_))
    ));
}

#[test]
fn module_runtime_rejects_duplicate_command_ids_within_module() {
    let module = TestModule::new(
        "welcome",
        vec![command("command.open"), command("command.open")],
    );

    let mut runtime = ModuleRuntime::default();
    let result = runtime.register(Box::new(module));
    assert!(matches!(
        result,
        Err(RegisterModuleError::DuplicateCommandId(_))
    ));
}

#[test]
fn module_runtime_rejects_duplicate_module_ids() {
    let first = SimpleModule::with_command("welcome", command("command.open"));
    let second = SimpleModule::with_command("welcome", command("command.close"));

    let mut runtime = ModuleRuntime::default();
    assert!(runtime.register(Box::new(first)).is_ok());
    let result = runtime.register(Box::new(second));
    assert!(matches!(
        result,
        Err(RegisterModuleError::DuplicateModuleId(_))
    ));
}

#[test]
fn module_runtime_register_uses_single_snapshot_for_stateful_module() {
    let mut runtime = ModuleRuntime::default();
    let first = SimpleModule::with_command("welcome", command("command.open"));
    assert!(runtime.register(Box::new(first)).is_ok());

    let stateful = StatefulModule::new(
        "stateful",
        vec![command("command.unique")],
        vec![command("command.open")],
    );
    assert!(runtime.register(Box::new(stateful)).is_ok());

    let duplicate_of_first_snapshot = SimpleModule::with_command("followup", command("command.unique"));
    let result = runtime.register(Box::new(duplicate_of_first_snapshot));
    assert!(matches!(
        result,
        Err(RegisterModuleError::DuplicateCommandId(_))
    ));
}

#[test]
fn module_runtime_rejects_empty_module_id() {
    let module = SimpleModule::with_command("", command("command.open"));
    let mut runtime = ModuleRuntime::default();

    let result = runtime.register(Box::new(module));
    assert!(matches!(
        result,
        Err(RegisterModuleError::InvalidModuleId(ModuleIdError::EmptyId))
    ));
}

#[test]
fn module_runtime_rejects_whitespace_only_module_id() {
    let module = SimpleModule::with_command("   ", command("command.open"));
    let mut runtime = ModuleRuntime::default();

    let result = runtime.register(Box::new(module));
    assert!(matches!(
        result,
        Err(RegisterModuleError::InvalidModuleId(ModuleIdError::EmptyId))
    ));
}

#[test]
fn command_contribution_rejects_empty_command_id() {
    let result = CommandContribution::try_new("   ");
    assert!(matches!(result, Err(CommandDescriptorError::EmptyId)));
}

#[test]
fn command_contribution_rejects_whitespace_in_command_id() {
    let result = CommandContribution::try_new("command open");
    assert!(matches!(
        result,
        Err(CommandDescriptorError::IdContainsWhitespace)
    ));
}
