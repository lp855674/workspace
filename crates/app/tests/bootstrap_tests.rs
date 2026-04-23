use app_ui::AppFrame;
use commands::CommandRegistry;
use module::ModuleRuntime;
use welcome::WelcomeModule;
use workspace::WorkspaceController;

fn build_bootstrap_parts() -> (WorkspaceController, ModuleRuntime, CommandRegistry) {
    let welcome = WelcomeModule::default();
    let mut module_runtime = ModuleRuntime::default();
    module_runtime.register(Box::new(welcome)).unwrap();

    let mut command_registry = CommandRegistry::default();
    for retained_module in module_runtime.retained_modules() {
        for command_id in retained_module.command_ids() {
            let command =
                commands::CommandDescriptor::try_new(command_id.as_str(), "Open").unwrap();
            command_registry.register(command).unwrap();
        }
    }

    let mut workspace = WorkspaceController::new("test-session");
    for panel in module_runtime.panels() {
        workspace.register_panel(panel.clone());
    }
    workspace.focus_panel("welcome.panel");

    (workspace, module_runtime, command_registry)
}

#[test]
fn bootstrap_registers_welcome_module_and_panel() {
    let (workspace, module_runtime, command_registry) = build_bootstrap_parts();

    assert_eq!(module_runtime.retained_module_count(), 1);
    assert_eq!(
        workspace.state().focused_panel.as_deref(),
        Some("welcome.panel")
    );
    assert_eq!(command_registry.len(), 1);
}

#[test]
fn frame_can_be_built_from_bootstrap_workspace() {
    let (workspace, _, _) = build_bootstrap_parts();
    let frame = AppFrame::new(workspace);
    assert_eq!(frame.title_text(), AppFrame::title());
}
