use crate::ToolbarButton;

pub enum MenuState {
    Keep,
    Update { active_buttons: Vec<ToolbarButton> },
}

impl MenuState {
    pub fn from(inner: wysiwyg::MenuState) -> Self {
        match inner {
            wysiwyg::MenuState::Keep => Self::Keep,
            wysiwyg::MenuState::Update(menu_update) => Self::Update {
                active_buttons: menu_update
                    .active_buttons
                    .iter()
                    .map(|button| ToolbarButton::from(button.clone()))
                    .collect(),
            },
        }
    }
}
