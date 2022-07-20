use std::sync::{Arc, Mutex};

use crate::ffi_action_response::ActionResponse;
use crate::ffi_composer_update::ComposerUpdate;

pub struct ComposerModel {
    inner: Mutex<wysiwyg::ComposerModel<u16>>,
}

impl ComposerModel {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(wysiwyg::ComposerModel::new()),
        }
    }

    pub fn select(
        self: &Arc<Self>,
        start_utf16_codeunit: u32,
        end_utf16_codeunit: u32,
    ) {
        let start = wysiwyg::Location::from(
            usize::try_from(start_utf16_codeunit).unwrap(),
        );
        let end = wysiwyg::Location::from(
            usize::try_from(end_utf16_codeunit).unwrap(),
        );

        self.inner.lock().unwrap().select(start, end);
    }

    pub fn replace_text(
        self: &Arc<Self>,
        new_text: String,
    ) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner
                .lock()
                .unwrap()
                .replace_text(&new_text.encode_utf16().collect::<Vec<_>>()),
        ))
    }

    pub fn replace_text_in(
        self: &Arc<Self>,
        new_text: String,
        start: u64,
        end: u64,
    ) -> Arc<ComposerUpdate> {
        let startSize = usize::try_from(start).unwrap();
        let endSize = usize::try_from(end).unwrap();
        Arc::new(ComposerUpdate::from(
            self.inner
                .lock()
                .unwrap()
                .replace_text_in(
                    &new_text.encode_utf16().collect::<Vec<_>>(),
                    startSize,
                    endSize,
                ),
        ))
    }

    pub fn backspace(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().backspace()))
    }

    pub fn delete(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().delete()))
    }

    pub fn delete_in(self: &Arc<Self>, start: u64, end: u64) -> Arc<ComposerUpdate> {
        let startSize = usize::try_from(start).unwrap();
        let endSize = usize::try_from(end).unwrap();
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().delete_in(startSize, endSize)))
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

    pub fn enter(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().enter()))
    }

    pub fn bold(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().bold()))
    }

    pub fn dump_contents(self: &Arc<Self>) -> Vec<u16> {
        self.inner.lock().unwrap().dump_contents()
    }
}
