use std::sync::{Arc, Mutex};

use crate::ffi_action_response::ActionResponse;
use crate::ffi_composer_update::ComposerUpdate;

pub struct ComposerModel {
    inner: Mutex<wysiwyg::ComposerModel>,
}

impl ComposerModel {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(wysiwyg::ComposerModel::new()),
        }
    }

    pub fn select(self: &Arc<Self>, start_codepoint: u32, end_codepoint: u32) {
        self.inner.lock().unwrap().select(
            usize::try_from(start_codepoint).unwrap(),
            usize::try_from(end_codepoint).unwrap(),
        )
    }

    pub fn replace_text(
        self: &Arc<Self>,
        new_text: String,
    ) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().replace_text(&new_text),
        ))
    }

    pub fn backspace(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().backspace()))
    }

    pub fn delete(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().delete()))
    }

    pub fn action_response(
        self: &Arc<Self>,
        action_id: String,
        response: ActionResponse,
    ) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner
                .lock()
                .unwrap()
                .action_response(action_id, response.into()),
        ))
    }
}
