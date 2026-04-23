#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WelcomePanelState {
    pub mode: String,
}

impl Default for WelcomePanelState {
    fn default() -> Self {
        Self {
            mode: "default".to_owned(),
        }
    }
}
