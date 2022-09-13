use crate::ComposerAction;

pub enum MenuState {
    Keep,
    Update {
        reversed_actions: Vec<ComposerAction>,
        disabled_actions: Vec<ComposerAction>,
    },
}

impl MenuState {
    pub fn from(inner: wysiwyg::MenuState) -> Self {
        match inner {
            wysiwyg::MenuState::Keep => Self::Keep,
            wysiwyg::MenuState::Update(menu_update) => Self::Update {
                reversed_actions: menu_update
                    .reversed_actions
                    .iter()
                    .map(|button| ComposerAction::from(button.clone()))
                    .collect(),
                disabled_actions: menu_update
                    .disabled_actions
                    .iter()
                    .map(|button| ComposerAction::from(button.clone()))
                    .collect(),
            },
        }
    }
}
