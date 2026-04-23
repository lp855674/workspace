#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PanelDescriptorError {
    EmptyId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct PanelTypeId(String);

impl PanelTypeId {
    pub fn try_new(value: impl Into<String>) -> Result<Self, PanelDescriptorError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(PanelDescriptorError::EmptyId);
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn from_static(value: &'static str) -> Self {
        Self(value.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PartialEq<&str> for PanelTypeId {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct PanelInstanceId(String);

impl PanelInstanceId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct PanelInstanceKey(String);

impl PanelInstanceKey {
    pub fn from_static(value: &'static str) -> Self {
        Self(value.to_owned())
    }

    pub fn from_string(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelMultiplicity {
    Singleton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelCloseBehavior {
    Hide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockPlacement {
    Left,
    Center,
    Bottom,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelDescriptor {
    pub id: PanelTypeId,
    pub title: String,
    pub dock: DockPlacement,
    pub multiplicity: PanelMultiplicity,
    pub close_behavior: PanelCloseBehavior,
    pub default_instance_key: PanelInstanceKey,
    pub restorable: bool,
}

impl PanelDescriptor {
    pub fn new(id: impl Into<String>, title: impl Into<String>, dock: DockPlacement) -> Self {
        let raw_id = id.into();
        let panel_type_id = PanelTypeId::try_new(raw_id)
            .unwrap_or_else(|_| panic!("panel descriptor id must not be empty"));

        Self {
            default_instance_key: PanelInstanceKey::from_string(panel_type_id.as_str().to_owned()),
            id: panel_type_id,
            title: title.into(),
            dock,
            multiplicity: PanelMultiplicity::Singleton,
            close_behavior: PanelCloseBehavior::Hide,
            restorable: true,
        }
    }

    pub fn singleton(id: PanelTypeId, title: impl Into<String>, dock: DockPlacement) -> Self {
        let default_instance_key = PanelInstanceKey::from_string(id.as_str().to_owned());

        Self {
            id,
            title: title.into(),
            dock,
            multiplicity: PanelMultiplicity::Singleton,
            close_behavior: PanelCloseBehavior::Hide,
            default_instance_key,
            restorable: true,
        }
    }

    pub fn multiplicity(&self) -> PanelMultiplicity {
        self.multiplicity
    }

    pub fn close_behavior(&self) -> PanelCloseBehavior {
        self.close_behavior
    }

    pub fn default_instance_key(&self) -> &PanelInstanceKey {
        &self.default_instance_key
    }
}
