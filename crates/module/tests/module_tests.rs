use module::{CommandContribution, FeatureModule, ModuleRuntime, SimpleModule};

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

#[test]
fn module_runtime_rejects_duplicate_command_ids_across_modules() {
    let first = SimpleModule::with_command("welcome", CommandContribution::new("command.open"));
    let second = SimpleModule::with_command("inspector", CommandContribution::new("command.open"));

    let mut runtime = ModuleRuntime::default();
    assert!(runtime.register(Box::new(first)).is_ok());
    assert!(runtime.register(Box::new(second)).is_err());
}

#[test]
fn module_runtime_rejects_duplicate_command_ids_within_module() {
    let module = TestModule::new(
        "welcome",
        vec![
            CommandContribution::new("command.open"),
            CommandContribution::new("command.open"),
        ],
    );

    let mut runtime = ModuleRuntime::default();
    assert!(runtime.register(Box::new(module)).is_err());
}

#[test]
fn module_runtime_rejects_duplicate_module_ids() {
    let first = SimpleModule::with_command("welcome", CommandContribution::new("command.open"));
    let second = SimpleModule::with_command("welcome", CommandContribution::new("command.close"));

    let mut runtime = ModuleRuntime::default();
    assert!(runtime.register(Box::new(first)).is_ok());
    assert!(runtime.register(Box::new(second)).is_err());
}
