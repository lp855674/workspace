use app_ui::AppFrame;
use commands::CommandRegistry;
use gpui::{
    Action, App, AppContext, Application, AssetSource, Menu, MenuItem, OsAction, Result,
    SharedString, WindowBounds,
    WindowDecorations, WindowOptions, actions, px, size,
};
use gpui_component::{Root, TitleBar};
use module::ModuleRuntime;
use std::borrow::Cow;
use std::fs;
use std::path::PathBuf;
use terminal::terminal_panel_descriptor;
use welcome::WelcomeModule;
use workspace::WorkspaceController;

actions!(
    app_menu,
    [
        AboutFramework,
        QuitApp,
        NewWorkspace,
        OpenWorkspace,
        SaveWorkspace,
        CloseEditor,
        UndoAction,
        RedoAction,
        CommandPalette,
        ToggleLeftDock,
        ToggleRightDock,
        ToggleBottomDock,
        FocusEditor,
        Documentation
    ]
);

pub fn boot_message() -> String {
    format!("starting {}", AppFrame::title())
}

struct AppBootstrap {
    frame: AppFrame,
    module_runtime: ModuleRuntime,
    command_registry: CommandRegistry,
}

struct WorkspaceAssets {
    base: PathBuf,
}

impl WorkspaceAssets {
    fn new() -> Self {
        Self {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets"),
        }
    }

    fn load_fonts(&self, cx: &App) -> Result<()> {
        let font_paths = self.list("fonts")?;
        let mut embedded_fonts = Vec::new();
        for font_path in font_paths {
            if font_path.ends_with(".ttf") {
                let font_bytes = cx
                    .asset_source()
                    .load(&font_path)?
                    .expect("workspace assets should return font bytes");
                embedded_fonts.push(font_bytes);
            }
        }
        cx.text_system().add_fonts(embedded_fonts)
    }
}

impl AssetSource for WorkspaceAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match fs::read(self.base.join(path)) {
            Ok(data) => Ok(Some(Cow::Owned(data))),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        match fs::read_dir(self.base.join(path)) {
            Ok(entries) => Ok(entries
                .filter_map(|entry| {
                    entry
                        .ok()
                        .and_then(|entry| entry.file_name().into_string().ok())
                        .map(SharedString::from)
                })
                .collect()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(error.into()),
        }
    }
}

fn build_initial_frame() -> AppBootstrap {
    let welcome = WelcomeModule::default();
    let mut module_runtime = ModuleRuntime::default();
    module_runtime
        .register(Box::new(welcome))
        .expect("registering built-in welcome module should succeed");

    let mut command_registry = CommandRegistry::default();
    for retained_module in module_runtime.retained_modules() {
        for command in retained_module.commands() {
            command_registry
                .register(command.clone())
                .expect("built-in command ids should be unique");
        }
    }

    let mut workspace = WorkspaceController::new("default-session");
    for panel in module_runtime.panels() {
        workspace.register_panel(panel.clone());
    }
    workspace.register_panel(terminal_panel_descriptor());
    for command in command_registry.commands() {
        workspace.record_command(command.id().as_str());
        if command.id().as_str() == "welcome.open" {
            workspace.dispatch(command.action().clone());
        }
    }

    AppBootstrap {
        frame: AppFrame::new(workspace),
        module_runtime,
        command_registry,
    }
}

fn install_menus(cx: &mut App) {
    cx.on_action(quit_app);
    cx.on_action(noop::<AboutFramework>);
    cx.on_action(noop::<NewWorkspace>);
    cx.on_action(noop::<OpenWorkspace>);
    cx.on_action(noop::<SaveWorkspace>);
    cx.on_action(noop::<CloseEditor>);
    cx.on_action(noop::<UndoAction>);
    cx.on_action(noop::<RedoAction>);
    cx.on_action(noop::<CommandPalette>);
    cx.on_action(noop::<ToggleLeftDock>);
    cx.on_action(noop::<ToggleRightDock>);
    cx.on_action(noop::<ToggleBottomDock>);
    cx.on_action(noop::<FocusEditor>);
    cx.on_action(noop::<Documentation>);

    cx.set_menus(vec![
        Menu {
            name: "Zed".into(),
            items: vec![
                MenuItem::action("About Framework", AboutFramework),
                MenuItem::separator(),
                MenuItem::action("Quit", QuitApp),
            ],
        },
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New Workspace", NewWorkspace),
                MenuItem::action("Open Workspace...", OpenWorkspace),
                MenuItem::separator(),
                MenuItem::action("Save Workspace", SaveWorkspace),
                MenuItem::action("Close Editor", CloseEditor),
            ],
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::action("Undo", UndoAction),
                MenuItem::action("Redo", RedoAction),
                MenuItem::separator(),
                MenuItem::os_action("Cut", CloseEditor, OsAction::Cut),
                MenuItem::os_action("Copy", FocusEditor, OsAction::Copy),
                MenuItem::os_action("Paste", OpenWorkspace, OsAction::Paste),
                MenuItem::os_action("Select All", SaveWorkspace, OsAction::SelectAll),
            ],
        },
        Menu {
            name: "View".into(),
            items: vec![
                MenuItem::action("Command Palette", CommandPalette),
                MenuItem::separator(),
                MenuItem::action("Toggle Left Dock", ToggleLeftDock),
                MenuItem::action("Toggle Right Dock", ToggleRightDock),
                MenuItem::action("Toggle Bottom Panel", ToggleBottomDock),
                MenuItem::action("Focus Editor", FocusEditor),
            ],
        },
        Menu {
            name: "Help".into(),
            items: vec![MenuItem::action("Documentation", Documentation)],
        },
    ]);
}

fn noop<T: Action>(_: &T, _: &mut App) {}

fn quit_app(_: &QuitApp, cx: &mut App) {
    cx.quit();
}

fn main() {
    Application::new()
        .with_assets(WorkspaceAssets::new())
        .run(|cx: &mut App| {
            gpui_component::init(cx);
            WorkspaceAssets::new()
                .load_fonts(cx)
                .expect("loading embedded workspace fonts should succeed");
            install_menus(cx);

            let bootstrap = build_initial_frame();
            let window_bounds = WindowBounds::centered(size(px(1280.), px(800.)), cx);

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(window_bounds),
                    titlebar: Some(TitleBar::title_bar_options()),
                    app_id: Some("com.workspace.zed_workbench_kernel".to_owned()),
                    window_decorations: Some(WindowDecorations::Client),
                    is_movable: true,
                    focus: true,
                    ..Default::default()
                },
                move |window, cx| {
                    let _module_count = bootstrap.module_runtime.retained_module_count();
                    let _command_count = bootstrap.command_registry.len();
                    let view = cx.new(|_| bootstrap.frame);
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )
            .expect("opening app window should succeed");

            cx.activate(true);
        });
}
