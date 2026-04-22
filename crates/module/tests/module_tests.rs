use module::{CommandContribution, ModuleRuntime, SimpleModule};

#[test]
fn module_runtime_rejects_duplicate_command_contributions() {
    let first = SimpleModule::with_command("welcome", CommandContribution::new("command.open"));
    let second = SimpleModule::with_command("inspector", CommandContribution::new("command.open"));

    let mut runtime = ModuleRuntime::default();
    assert!(runtime.register(Box::new(first)).is_ok());
    assert!(runtime.register(Box::new(second)).is_err());
}
