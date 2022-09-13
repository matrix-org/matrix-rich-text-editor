use std::sync::{Arc, Mutex};

use widestring::Utf16String;

use crate::ffi_composer_state::ComposerState;
use crate::ffi_composer_update::ComposerUpdate;
use crate::ffi_format_type::InlineFormatType;

pub struct ComposerModel {
    inner: Mutex<wysiwyg::ComposerModel<Utf16String>>,
}

impl ComposerModel {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(wysiwyg::ComposerModel::new()),
        }
    }

    pub fn replace_all_html(
        self: &Arc<Self>,
        html: String,
    ) -> Arc<ComposerUpdate> {
        let html = Utf16String::from_str(&html);
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().replace_all_html(&html),
        ))
    }

    pub fn select(
        self: &Arc<Self>,
        start_utf16_codeunit: u32,
        end_utf16_codeunit: u32,
    ) -> Arc<ComposerUpdate> {
        let start = wysiwyg::Location::from(
            usize::try_from(start_utf16_codeunit).unwrap(),
        );
        let end = wysiwyg::Location::from(
            usize::try_from(end_utf16_codeunit).unwrap(),
        );

        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().select(start, end),
        ))
    }

    pub fn replace_text(
        self: &Arc<Self>,
        new_text: String,
    ) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner
                .lock()
                .unwrap()
                .replace_text(Utf16String::from_str(&new_text)),
        ))
    }

    pub fn replace_text_in(
        self: &Arc<Self>,
        new_text: String,
        start: u32,
        end: u32,
    ) -> Arc<ComposerUpdate> {
        let start = usize::try_from(start).unwrap();
        let end = usize::try_from(end).unwrap();
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().replace_text_in(
                Utf16String::from_str(&new_text),
                start,
                end,
            ),
        ))
    }

    pub fn backspace(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().backspace()))
    }

    pub fn delete(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().delete()))
    }

    pub fn delete_in(
        self: &Arc<Self>,
        start: u32,
        end: u32,
    ) -> Arc<ComposerUpdate> {
        let start = usize::try_from(start).unwrap();
        let end = usize::try_from(end).unwrap();
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().delete_in(start, end),
        ))
    }

    pub fn enter(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().enter()))
    }

    pub fn format(
        self: &Arc<Self>,
        format: InlineFormatType,
    ) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().format(format.into()),
        ))
    }

    pub fn ordered_list(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().ordered_list(),
        ))
    }

    pub fn unordered_list(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().unordered_list(),
        ))
    }

    pub fn undo(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().undo()))
    }

    pub fn redo(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().redo()))
    }

    pub fn set_link(self: &Arc<Self>, link: String) -> Arc<ComposerUpdate> {
        let link = Utf16String::from_str(&link);
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().set_link(link),
        ))
    }

    pub fn to_tree(self: &Arc<Self>) -> String {
        self.inner.lock().unwrap().to_tree().to_string()
    }

    pub fn dump_state(self: &Arc<Self>) -> ComposerState {
        self.inner
            .lock()
            .unwrap()
            .get_current_state()
            .clone()
            .into()
    }
}
