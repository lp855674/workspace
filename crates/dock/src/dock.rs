pub use panel::DockPlacement;
use panel::PanelInstanceKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisiblePanel {
    pub placement: DockPlacement,
    pub panel_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VisibleDockState {
    pub visible_panels: Vec<VisiblePanel>,
}

impl VisibleDockState {
    pub fn with_visible(placement: DockPlacement, panel_id: impl Into<String>) -> Self {
        Self {
            visible_panels: vec![VisiblePanel {
                placement,
                panel_id: panel_id.into(),
            }],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DockContainerState {
    pub tabs: Vec<PanelInstanceKey>,
    pub active: Option<PanelInstanceKey>,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DockLayoutState {
    left: DockContainerState,
    center: DockContainerState,
    bottom: DockContainerState,
    right: DockContainerState,
}

impl DockLayoutState {
    pub fn show_panel(&mut self, placement: DockPlacement, panel_key: PanelInstanceKey) {
        let container = self.container_mut(placement);
        if !container.tabs.iter().any(|existing| existing == &panel_key) {
            container.tabs.push(panel_key.clone());
        }
        container.active = Some(panel_key);
        container.visible = true;
    }

    pub fn move_panel(&mut self, panel_key: &PanelInstanceKey, new_placement: DockPlacement) {
        self.remove_panel(panel_key);
        self.show_panel(new_placement, panel_key.clone());
    }

    pub fn active_panel(&self, placement: DockPlacement) -> Option<&PanelInstanceKey> {
        self.container(placement).active.as_ref()
    }

    pub fn is_visible(&self, placement: DockPlacement) -> bool {
        self.container(placement).visible
    }

    fn remove_panel(&mut self, panel_key: &PanelInstanceKey) {
        for placement in [
            DockPlacement::Left,
            DockPlacement::Center,
            DockPlacement::Bottom,
            DockPlacement::Right,
        ] {
            let container = self.container_mut(placement);
            let original_len = container.tabs.len();
            container.tabs.retain(|existing| existing != panel_key);
            if original_len != container.tabs.len() {
                if container.active.as_ref() == Some(panel_key) {
                    container.active = container.tabs.last().cloned();
                }
                container.visible = !container.tabs.is_empty();
            }
        }
    }

    fn container(&self, placement: DockPlacement) -> &DockContainerState {
        match placement {
            DockPlacement::Left => &self.left,
            DockPlacement::Center => &self.center,
            DockPlacement::Bottom => &self.bottom,
            DockPlacement::Right => &self.right,
        }
    }

    fn container_mut(&mut self, placement: DockPlacement) -> &mut DockContainerState {
        match placement {
            DockPlacement::Left => &mut self.left,
            DockPlacement::Center => &mut self.center,
            DockPlacement::Bottom => &mut self.bottom,
            DockPlacement::Right => &mut self.right,
        }
    }
}
