use actions::{ActionEnvelope, PanelAction};
use commands::{CommandDescriptor, CommandDescriptorError};
use module::{
    CommandContribution, FeatureModule, ModuleIdError, ModuleRuntime, PanelContribution,
    RegisterModuleError, SimpleModule,
};
use panel::{DockPlacement, PanelDescriptor};
use std::cell::Cell;
use std::rc::Rc;

fn command(command_id: &str) -> CommandContribution {
    CommandContribution::try_new(command_id).unwrap()
}

fn command_with_panel_action(command_id: &str, panel_type_id: &str) -> CommandContribution {
    let descriptor = CommandDescriptor::try_new_with_action(
        command_id,
        "Open Panel",
        ActionEnvelope::panel(PanelAction::Open {
            panel_type_id: panel_type_id.to_owned(),
        }),
    )
    .unwrap();
    CommandContribution::new(descriptor)
}

fn panel(panel_id: &str) -> PanelContribution {
    PanelContribution::new(PanelDescriptor::new(panel_id, "Panel", DockPlacement::Left))
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

struct ToggleableModule {
    use_second_snapshot: Rc<Cell<bool>>,
    first_module_id: String,
    second_module_id: String,
    first_commands: Vec<CommandContribution>,
    second_commands: Vec<CommandContribution>,
}

impl ToggleableModule {
    fn new(
        use_second_snapshot: Rc<Cell<bool>>,
        first_module_id: impl Into<String>,
        second_module_id: impl Into<String>,
        first_commands: Vec<CommandContribution>,
        second_commands: Vec<CommandContribution>,
    ) -> Self {
        Self {
            use_second_snapshot,
            first_module_id: first_module_id.into(),
            second_module_id: second_module_id.into(),
            first_commands,
            second_commands,
        }
    }
}

struct DropObservedModule {
    module_id: String,
    commands: Vec<CommandContribution>,
    drop_count: Rc<Cell<usize>>,
}

impl DropObservedModule {
    fn new(
        module_id: impl Into<String>,
        commands: Vec<CommandContribution>,
        drop_count: Rc<Cell<usize>>,
    ) -> Self {
        Self {
            module_id: module_id.into(),
            commands,
            drop_count,
        }
    }
}

impl FeatureModule for ToggleableModule {
    fn module_id(&self) -> &str {
        if self.use_second_snapshot.get() {
            &self.second_module_id
        } else {
            &self.first_module_id
        }
    }

    fn command_contributions(&self) -> &[CommandContribution] {
        if self.use_second_snapshot.get() {
            &self.second_commands
        } else {
            &self.first_commands
        }
    }
}

impl FeatureModule for DropObservedModule {
    fn module_id(&self) -> &str {
        &self.module_id
    }

    fn command_contributions(&self) -> &[CommandContribution] {
        &self.commands
    }
}

impl Drop for DropObservedModule {
    fn drop(&mut self) {
        self.drop_count.set(self.drop_count.get() + 1);
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
fn module_runtime_retains_action_carrying_command_descriptors() {
    let module = SimpleModule::with_command(
        "welcome",
        command_with_panel_action("welcome.open", "welcome.panel"),
    );

    let mut runtime = ModuleRuntime::default();
    assert!(runtime.register(Box::new(module)).is_ok());

    let registered = runtime
        .retained_modules()
        .first()
        .expect("registered module should be retained");
    assert_eq!(registered.command_ids()[0].as_str(), "welcome.open");
    assert_eq!(registered.commands()[0].id().as_str(), "welcome.open");
    assert_eq!(registered.commands()[0].action().kind(), "panel");
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

    let duplicate_of_first_snapshot =
        SimpleModule::with_command("followup", command("command.unique"));
    let result = runtime.register(Box::new(duplicate_of_first_snapshot));
    assert!(matches!(
        result,
        Err(RegisterModuleError::DuplicateCommandId(_))
    ));
}

#[test]
fn module_runtime_retains_canonical_module_id_snapshot() {
    let mut runtime = ModuleRuntime::default();
    let first = SimpleModule::with_command("  welcome  ", command("command.open"));
    assert!(runtime.register(Box::new(first)).is_ok());

    assert_eq!(runtime.retained_modules().len(), 1);
    assert_eq!(
        runtime.retained_modules()[0].module_id().as_str(),
        "welcome"
    );

    let duplicate = SimpleModule::with_command("welcome", command("command.close"));
    let result = runtime.register(Box::new(duplicate));
    match result {
        Err(RegisterModuleError::DuplicateModuleId(module_id)) => {
            assert_eq!(module_id.as_str(), "welcome");
        }
        other => panic!("expected duplicate module id error, got {other:?}"),
    }
}

#[test]
fn module_runtime_retained_metadata_is_immutable_after_registration() {
    let mut runtime = ModuleRuntime::default();
    let use_second_snapshot = Rc::new(Cell::new(false));
    let mutable_module = ToggleableModule::new(
        Rc::clone(&use_second_snapshot),
        "alpha",
        "beta",
        vec![command("command.alpha")],
        vec![command("command.beta")],
    );

    assert!(runtime.register(Box::new(mutable_module)).is_ok());
    assert_eq!(runtime.retained_modules().len(), 1);
    assert_eq!(runtime.retained_modules()[0].module_id().as_str(), "alpha");
    assert_eq!(
        runtime.retained_modules()[0].command_ids()[0].as_str(),
        "command.alpha"
    );

    use_second_snapshot.set(true);
    assert_eq!(runtime.retained_modules()[0].module_id().as_str(), "alpha");
    assert_eq!(
        runtime.retained_modules()[0].command_ids()[0].as_str(),
        "command.alpha"
    );

    let duplicate_initial = SimpleModule::with_command("alpha", command("command.gamma"));
    assert!(matches!(
        runtime.register(Box::new(duplicate_initial)),
        Err(RegisterModuleError::DuplicateModuleId(_))
    ));

    let duplicate_initial_command = SimpleModule::with_command("gamma", command("command.alpha"));
    assert!(matches!(
        runtime.register(Box::new(duplicate_initial_command)),
        Err(RegisterModuleError::DuplicateCommandId(_))
    ));

    let second_snapshot_values = SimpleModule::with_command("beta", command("command.beta"));
    assert!(runtime.register(Box::new(second_snapshot_values)).is_ok());
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
fn module_runtime_retains_registered_modules_only_after_successful_validation() {
    let mut runtime = ModuleRuntime::default();
    assert_eq!(runtime.retained_module_count(), 0);

    let first = SimpleModule::with_command("welcome", command("command.open"));
    assert!(runtime.register(Box::new(first)).is_ok());
    assert_eq!(runtime.retained_module_count(), 1);

    let duplicate_command = SimpleModule::with_command("inspector", command("command.open"));
    assert!(matches!(
        runtime.register(Box::new(duplicate_command)),
        Err(RegisterModuleError::DuplicateCommandId(_))
    ));
    assert_eq!(runtime.retained_module_count(), 1);

    let second = SimpleModule::with_command("settings", command("command.close"));
    assert!(runtime.register(Box::new(second)).is_ok());
    assert_eq!(runtime.retained_module_count(), 2);
}

#[test]
fn module_runtime_keeps_registered_module_alive_until_runtime_drop() {
    let drop_count = Rc::new(Cell::new(0));

    {
        let mut runtime = ModuleRuntime::default();
        let module = DropObservedModule::new(
            "welcome",
            vec![command("command.open")],
            Rc::clone(&drop_count),
        );

        assert!(runtime.register(Box::new(module)).is_ok());
        assert_eq!(drop_count.get(), 0);
        assert_eq!(runtime.retained_module_count(), 1);
    }

    assert_eq!(drop_count.get(), 1);
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

#[test]
fn module_runtime_rejects_duplicate_panel_ids_across_modules() {
    let first = SimpleModule::with_command_and_panel(
        "welcome",
        command("command.open"),
        panel("panel.shared"),
    );
    let second = SimpleModule::with_command_and_panel(
        "inspector",
        command("command.close"),
        panel("panel.shared"),
    );

    let mut runtime = ModuleRuntime::default();
    assert!(runtime.register(Box::new(first)).is_ok());
    assert!(matches!(
        runtime.register(Box::new(second)),
        Err(RegisterModuleError::DuplicatePanelId(_))
    ));
}
