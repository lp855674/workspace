use actions::{ActionEnvelope, PanelActionEnvelope};
use command_palette::CommandPaletteState;
use std::collections::BTreeMap;

use dock::{DockLayoutState, DockPlacement, VisibleDockState, VisiblePanel};
use panel::{PanelDescriptor, PanelInstanceId, PanelInstanceKey};

pub const SUPPORTED_WORKSPACE_SESSION_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceSession {
    pub session_id: String,
    pub restored: bool,
}

impl WorkspaceSession {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            restored: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerializedDockPanel {
    pub panel_id: String,
    pub placement: DockPlacement,
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SerializedWorkspaceState {
    pub visible_panels: Vec<String>,
    pub focused_panel: Option<String>,
    pub dock_panels: Vec<SerializedDockPanel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerializedPanelState {
    pub panel_type_id: String,
    pub panel_instance_key: String,
    pub placement: DockPlacement,
    pub visible: bool,
    pub focused: bool,
    pub panel_state_json: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerializedWorkspaceSession {
    pub schema_version: u32,
    pub panels: Vec<SerializedPanelState>,
}

impl Default for SerializedWorkspaceSession {
    fn default() -> Self {
        Self {
            schema_version: SUPPORTED_WORKSPACE_SESSION_SCHEMA_VERSION,
            panels: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PanelRuntimeState {
    pub descriptor: PanelDescriptor,
    pub instance_id: PanelInstanceId,
    pub instance_key: PanelInstanceKey,
    pub placement: DockPlacement,
    pub visible: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorkspaceState {
    pub registered_panels: BTreeMap<String, PanelDescriptor>,
    pub live_panels: BTreeMap<String, PanelRuntimeState>,
    pub dock_layout: DockLayoutState,
    pub visible_panels: Vec<String>,
    pub focused_panel: Option<String>,
    pub dock_state: VisibleDockState,
    pub recent_commands: Vec<String>,
    pub command_palette: CommandPaletteState,
}

impl WorkspaceState {
    pub fn register_panel(&mut self, descriptor: PanelDescriptor) {
        self.registered_panels
            .insert(descriptor.id.as_str().to_owned(), descriptor);
    }

    pub fn dispatch(&mut self, action: &ActionEnvelope) {
        match action {
            ActionEnvelope::Panel(PanelActionEnvelope {
                action,
                panel_type_id,
            }) if action == "open" => {
                self.open_panel(panel_type_id);
            }
            ActionEnvelope::Panel(PanelActionEnvelope {
                action,
                panel_type_id,
            }) if action == "toggle" => {
                if self.is_panel_visible(panel_type_id) {
                    self.hide_panel(panel_type_id);
                } else {
                    self.open_panel(panel_type_id);
                }
            }
            ActionEnvelope::Panel(PanelActionEnvelope {
                action,
                panel_type_id,
            }) if action == "focus" => {
                if self.is_panel_visible(panel_type_id) {
                    self.focused_panel = Some(panel_type_id.to_owned());
                } else {
                    self.open_panel(panel_type_id);
                }
            }
            ActionEnvelope::Panel(PanelActionEnvelope {
                action,
                panel_type_id,
            }) if action == "close" => {
                self.hide_panel(panel_type_id);
            }
            _ => {}
        }
    }

    pub fn open_panel(&mut self, panel_type_id: &str) {
        self.open_panel_at(panel_type_id, None);
    }

    fn open_panel_at(&mut self, panel_type_id: &str, placement: Option<DockPlacement>) {
        let Some(descriptor) = self.registered_panels.get(panel_type_id).cloned() else {
            return;
        };
        let placement = placement.unwrap_or(descriptor.dock);

        let runtime = self
            .live_panels
            .entry(panel_type_id.to_owned())
            .or_insert_with(|| PanelRuntimeState {
                descriptor: descriptor.clone(),
                instance_id: PanelInstanceId::new(format!("{panel_type_id}#singleton")),
                instance_key: descriptor.default_instance_key().clone(),
                placement,
                visible: false,
            });

        runtime.placement = placement;
        runtime.visible = true;
        let instance_key = runtime.instance_key.clone();
        self.add_visible_panel(panel_type_id, placement);
        self.dock_layout.show_panel(placement, instance_key);
        self.focused_panel = Some(panel_type_id.to_owned());
    }

    pub fn show_panel(&mut self, panel_id: &str) {
        if self.registered_panels.contains_key(panel_id) {
            self.open_panel(panel_id);
            return;
        }

        self.show_legacy_panel(panel_id, DockPlacement::Center);
    }

    pub fn hide_panel(&mut self, panel_type_id: &str) {
        if let Some(runtime) = self.live_panels.get_mut(panel_type_id) {
            runtime.visible = false;
        }
        self.visible_panels
            .retain(|existing| existing != panel_type_id);
        self.dock_state
            .visible_panels
            .retain(|panel| panel.panel_id != panel_type_id);
        if self.focused_panel.as_deref() == Some(panel_type_id) {
            self.focused_panel = None;
        }
        self.rebuild_dock_layout_from_live_panels();
    }

    pub fn is_panel_visible(&self, panel_type_id: &str) -> bool {
        self.live_panels
            .get(panel_type_id)
            .map(|runtime| runtime.visible)
            .unwrap_or_else(|| {
                self.visible_panels
                    .iter()
                    .any(|existing| existing == panel_type_id)
            })
    }

    pub fn visible_panels(&self) -> &[String] {
        &self.visible_panels
    }

    fn add_visible_panel(&mut self, panel_id: &str, placement: DockPlacement) {
        if !self
            .visible_panels
            .iter()
            .any(|existing| existing == panel_id)
        {
            self.visible_panels.push(panel_id.to_owned());
        }

        if self
            .dock_state
            .visible_panels
            .iter()
            .any(|panel| panel.panel_id == panel_id)
        {
            return;
        }

        self.dock_state.visible_panels.push(VisiblePanel {
            placement,
            panel_id: panel_id.to_owned(),
        });
    }

    fn show_legacy_panel(&mut self, panel_id: &str, placement: DockPlacement) {
        self.add_visible_panel(panel_id, placement);
    }

    pub fn focus_panel(&mut self, panel_id: &str) {
        self.show_panel(panel_id);
        self.focused_panel = Some(panel_id.to_owned());
    }

    pub fn record_command(&mut self, command_id: &str) {
        self.recent_commands.push(command_id.to_owned());
    }

    pub fn toggle_command_palette(&mut self) {
        self.command_palette.toggle();
    }

    pub fn serialize(&self) -> SerializedWorkspaceState {
        SerializedWorkspaceState {
            visible_panels: self.visible_panels.clone(),
            focused_panel: self.focused_panel.clone(),
            dock_panels: self
                .dock_state
                .visible_panels
                .iter()
                .map(|panel| SerializedDockPanel {
                    panel_id: panel.panel_id.clone(),
                    placement: panel.placement,
                })
                .collect(),
        }
    }

    pub fn serialize_session(&self) -> SerializedWorkspaceSession {
        SerializedWorkspaceSession {
            schema_version: SUPPORTED_WORKSPACE_SESSION_SCHEMA_VERSION,
            panels: self
                .live_panels
                .iter()
                .map(|(panel_type_id, runtime)| SerializedPanelState {
                    panel_type_id: panel_type_id.clone(),
                    panel_instance_key: panel_type_id.clone(),
                    placement: runtime.placement,
                    visible: runtime.visible,
                    focused: self.focused_panel.as_deref() == Some(panel_type_id.as_str()),
                    panel_state_json: None,
                })
                .collect(),
        }
    }

    pub fn restore(&mut self, state: &SerializedWorkspaceState) {
        self.visible_panels = state.visible_panels.clone();
        self.focused_panel = state.focused_panel.clone();
        self.dock_state = VisibleDockState {
            visible_panels: state
                .dock_panels
                .iter()
                .map(|panel| VisiblePanel {
                    placement: panel.placement,
                    panel_id: panel.panel_id.clone(),
                })
                .collect(),
        };
    }

    pub fn restore_versioned_session(&mut self, session: &SerializedWorkspaceSession) {
        self.live_panels.clear();
        self.visible_panels.clear();
        self.focused_panel = None;
        self.dock_state = VisibleDockState::default();
        self.dock_layout = DockLayoutState::default();

        if session.schema_version != SUPPORTED_WORKSPACE_SESSION_SCHEMA_VERSION {
            return;
        }

        for panel in &session.panels {
            if !self.registered_panels.contains_key(&panel.panel_type_id) {
                continue;
            }

            if panel.visible || panel.focused {
                self.open_panel_at(&panel.panel_type_id, Some(panel.placement));
            }

            if panel.focused && self.is_panel_visible(&panel.panel_type_id) {
                self.focused_panel = Some(panel.panel_type_id.clone());
            }
        }
    }

    fn rebuild_dock_layout_from_live_panels(&mut self) {
        self.dock_layout = DockLayoutState::default();
        for runtime in self.live_panels.values() {
            if runtime.visible {
                self.dock_layout
                    .show_panel(runtime.placement, runtime.instance_key.clone());
            }
        }
    }
}

pub trait WorkspaceRestoreSource {
    fn restore_into(&self, state: &mut WorkspaceState);
}

impl WorkspaceRestoreSource for SerializedWorkspaceState {
    fn restore_into(&self, state: &mut WorkspaceState) {
        state.restore(self);
    }
}

impl WorkspaceRestoreSource for SerializedWorkspaceSession {
    fn restore_into(&self, state: &mut WorkspaceState) {
        state.restore_versioned_session(self);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceController {
    session: WorkspaceSession,
    state: WorkspaceState,
}

impl WorkspaceController {
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session: WorkspaceSession::new(session_id),
            state: WorkspaceState::default(),
        }
    }

    pub fn from_state(session_id: impl Into<String>, state: WorkspaceState) -> Self {
        Self {
            session: WorkspaceSession::new(session_id),
            state,
        }
    }

    pub fn register_panel(&mut self, descriptor: PanelDescriptor) {
        self.state.register_panel(descriptor);
    }

    pub fn show_panel(&mut self, panel_id: &str) {
        self.state.show_panel(panel_id);
    }

    pub fn focus_panel(&mut self, panel_id: &str) {
        self.state.focus_panel(panel_id);
    }

    pub fn record_command(&mut self, command_id: &str) {
        self.state.record_command(command_id);
    }

    pub fn toggle_command_palette(&mut self) {
        self.state.toggle_command_palette();
    }

    pub fn dispatch(&mut self, action: ActionEnvelope) {
        self.state.dispatch(&action);
    }

    pub fn restore_session<T>(&mut self, state: &T)
    where
        T: WorkspaceRestoreSource,
    {
        self.session.restored = true;
        state.restore_into(&mut self.state);
    }

    pub fn serialize(&self) -> SerializedWorkspaceState {
        self.state.serialize()
    }

    pub fn serialize_session(&self) -> SerializedWorkspaceSession {
        self.state.serialize_session()
    }

    pub fn session(&self) -> &WorkspaceSession {
        &self.session
    }

    pub fn state(&self) -> &WorkspaceState {
        &self.state
    }
}
