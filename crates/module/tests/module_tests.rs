use module::{CommandContribution, FeatureModule, ModuleRuntime, SimpleModule};
use std::cell::Cell;

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

#[test]
fn module_runtime_register_uses_single_snapshot_for_stateful_module() {
    let mut runtime = ModuleRuntime::default();
    let first = SimpleModule::with_command("welcome", CommandContribution::new("command.open"));
    assert!(runtime.register(Box::new(first)).is_ok());

    let stateful = StatefulModule::new(
        "stateful",
        vec![CommandContribution::new("command.unique")],
        vec![CommandContribution::new("command.open")],
    );
    assert!(runtime.register(Box::new(stateful)).is_ok());

    let duplicate_of_first_snapshot =
        SimpleModule::with_command("followup", CommandContribution::new("command.unique"));
    assert!(runtime
        .register(Box::new(duplicate_of_first_snapshot))
        .is_err());
}
