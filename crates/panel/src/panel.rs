use dock::DockPlacement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelDescriptor {
    pub id: String,
    pub title: String,
    pub dock: DockPlacement,
    pub restorable: bool,
}

impl PanelDescriptor {
    pub fn new(id: impl Into<String>, title: impl Into<String>, dock: DockPlacement) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            dock,
            restorable: true,
        }
    }
}
