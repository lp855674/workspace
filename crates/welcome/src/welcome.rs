mod welcome_panel;

use actions::{ActionEnvelope, PanelAction};
use commands::CommandDescriptor;
use module::{CommandContribution, FeatureModule, PanelContribution, SimpleModule};
use panel::{DockPlacement, PanelDescriptor, PanelTypeId};

pub use welcome_panel::WelcomePanelState;

pub struct WelcomeModule {
    inner: SimpleModule,
    panel: PanelDescriptor,
}

impl WelcomeModule {
    pub fn new() -> Self {
        let panel = welcome_panel_descriptor();
        Self {
            inner: SimpleModule::with_command_and_panel(
                "welcome",
                welcome_command_contribution(),
                PanelContribution::new(panel.clone()),
            ),
            panel,
        }
    }

    pub fn panel(&self) -> &PanelDescriptor {
        &self.panel
    }
}

impl Default for WelcomeModule {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureModule for WelcomeModule {
    fn module_id(&self) -> &str {
        self.inner.module_id()
    }

    fn command_contributions(&self) -> &[CommandContribution] {
        self.inner.command_contributions()
    }

    fn panel_contributions(&self) -> &[PanelContribution] {
        self.inner.panel_contributions()
    }
}

fn welcome_command_contribution() -> CommandContribution {
    let command = match CommandDescriptor::try_new_with_action(
        "welcome.open",
        "Open Welcome",
        ActionEnvelope::panel(PanelAction::Open {
            panel_type_id: "welcome.panel".to_owned(),
        }),
    ) {
        Ok(command) => command,
        Err(error) => panic!("invalid built-in welcome command metadata: {error}"),
    };

    CommandContribution::new(command)
}

fn welcome_panel_descriptor() -> PanelDescriptor {
    let panel_type_id = match PanelTypeId::try_new("welcome.panel") {
        Ok(panel_type_id) => panel_type_id,
        Err(error) => panic!("invalid built-in welcome panel id: {error:?}"),
    };

    PanelDescriptor::singleton(panel_type_id, "Welcome", DockPlacement::Center)
}
