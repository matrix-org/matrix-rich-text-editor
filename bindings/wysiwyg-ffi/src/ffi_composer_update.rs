use std::sync::Arc;

use widestring::Utf16String;

use crate::ffi_composer_action::ComposerAction;
use crate::ffi_menu_state::MenuState;
use crate::ffi_text_update::TextUpdate;

pub struct ComposerUpdate {
    inner: wysiwyg::ComposerUpdate<Utf16String>,
}

impl ComposerUpdate {
    pub fn from(inner: wysiwyg::ComposerUpdate<Utf16String>) -> Self {
        Self { inner }
    }

    pub fn text_update(&self) -> TextUpdate {
        TextUpdate::from(self.inner.text_update.clone())
    }

    pub fn menu_state(&self) -> MenuState {
        MenuState::from(self.inner.menu_state.clone())
    }

    pub fn actions(&self) -> Vec<Arc<ComposerAction>> {
        self.inner
            .actions
            .iter()
            .map(|action| Arc::new(ComposerAction::from(action.clone())))
            .collect()
    }
}
