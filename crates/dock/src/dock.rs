pub use panel::DockPlacement;

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
