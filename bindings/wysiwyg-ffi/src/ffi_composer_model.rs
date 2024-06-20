use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::vec;

use widestring::Utf16String;

use crate::ffi_composer_state::ComposerState;
use crate::ffi_composer_update::ComposerUpdate;
use crate::ffi_dom_creation_error::DomCreationError;
use crate::ffi_link_actions::LinkAction;
use crate::ffi_mentions_state::MentionsState;
use crate::into_ffi::IntoFfi;
use crate::{ActionState, ComposerAction, SuggestionPattern};

#[derive(Default, uniffi::Object)]
pub struct ComposerModel {
    inner: Mutex<wysiwyg::ComposerModel<Utf16String>>,
}

impl ComposerModel {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(wysiwyg::ComposerModel::new()),
        }
    }
}

#[uniffi::export]
impl ComposerModel {
    pub fn set_content_from_html(
        self: &Arc<Self>,
        html: String,
    ) -> Result<Arc<ComposerUpdate>, DomCreationError> {
        let html = Utf16String::from_str(&html);
        let update = self.inner.lock().unwrap().set_content_from_html(&html)?;
        Ok(Arc::new(ComposerUpdate::from(update)))
    }

    pub fn set_content_from_markdown(
        self: &Arc<Self>,
        markdown: String,
    ) -> Result<Arc<ComposerUpdate>, DomCreationError> {
        let markdown = Utf16String::from_str(&markdown);
        let update = self
            .inner
            .lock()
            .unwrap()
            .set_content_from_markdown(&markdown)?;
        Ok(Arc::new(ComposerUpdate::from(update)))
    }

    pub fn get_content_as_html(self: &Arc<Self>) -> String {
        self.inner.lock().unwrap().get_content_as_html().to_string()
    }

    pub fn get_content_as_message_html(self: &Arc<Self>) -> String {
        self.inner
            .lock()
            .unwrap()
            .get_content_as_message_html()
            .to_string()
    }

    pub fn get_content_as_markdown(self: &Arc<Self>) -> String {
        self.inner
            .lock()
            .unwrap()
            .get_content_as_markdown()
            .to_string()
    }

    pub fn get_content_as_message_markdown(self: &Arc<Self>) -> String {
        self.inner
            .lock()
            .unwrap()
            .get_content_as_message_markdown()
            .to_string()
    }

    pub fn get_content_as_plain_text(self: &Arc<Self>) -> String {
        self.inner
            .lock()
            .unwrap()
            .get_content_as_plain_text()
            .to_string()
    }

    pub fn clear(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().clear()))
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

    pub fn replace_text_suggestion(
        self: &Arc<Self>,
        new_text: String,
        suggestion: SuggestionPattern,
    ) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().replace_text_suggestion(
                Utf16String::from_str(&new_text),
                wysiwyg::SuggestionPattern::from(suggestion),
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

    pub fn bold(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().bold()))
    }

    pub fn italic(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().italic()))
    }

    pub fn strike_through(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().strike_through(),
        ))
    }

    pub fn underline(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().underline()))
    }

    pub fn inline_code(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().inline_code(),
        ))
    }

    pub fn code_block(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().code_block(),
        ))
    }

    pub fn quote(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().quote()))
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

    pub fn set_link(
        self: &Arc<Self>,
        url: String,
        attributes: Vec<Attribute>,
    ) -> Arc<ComposerUpdate> {
        let url = Utf16String::from_str(&url);
        let attrs = attributes
            .iter()
            .map(|attr| {
                (
                    Utf16String::from_str(&attr.key),
                    Utf16String::from_str(&attr.value),
                )
            })
            .collect();
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().set_link(url, attrs),
        ))
    }

    pub fn set_link_with_text(
        self: &Arc<Self>,
        url: String,
        text: String,
        attributes: Vec<Attribute>,
    ) -> Arc<ComposerUpdate> {
        let url = Utf16String::from_str(&url);
        let text = Utf16String::from_str(&html_escape::encode_safe(&text));
        let attrs = attributes
            .iter()
            .map(|attr| {
                (
                    Utf16String::from_str(&attr.key),
                    Utf16String::from_str(&attr.value),
                )
            })
            .collect();
        Arc::new(ComposerUpdate::from(
            self.inner
                .lock()
                .unwrap()
                .set_link_with_text(url, text, attrs),
        ))
    }

    /// Creates an at-room mention node and inserts it into the composer at the current selection
    pub fn insert_at_room_mention(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().insert_at_room_mention(vec![]),
        ))
    }

    /// Creates a mention node and inserts it into the composer at the current selection
    pub fn insert_mention(
        self: &Arc<Self>,
        url: String,
        text: String,
        _attributes: Vec<Attribute>, // TODO remove attributes
    ) -> Arc<ComposerUpdate> {
        let url = Utf16String::from_str(&url);
        let text = Utf16String::from_str(&html_escape::encode_safe(&text));
        let attrs = vec![];
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().insert_mention(url, text, attrs),
        ))
    }

    /// Creates an at-room mention node and inserts it into the composer, replacing the
    /// text content defined by the suggestion
    pub fn insert_at_room_mention_at_suggestion(
        self: &Arc<Self>,
        suggestion: SuggestionPattern,
    ) -> Arc<ComposerUpdate> {
        let suggestion = wysiwyg::SuggestionPattern::from(suggestion);
        let attrs = vec![];
        Arc::new(ComposerUpdate::from(
            self.inner
                .lock()
                .unwrap()
                .insert_at_room_mention_at_suggestion(suggestion, attrs),
        ))
    }

    /// Creates a mention node and inserts it into the composer, replacing the
    /// text content defined by the suggestion
    pub fn insert_mention_at_suggestion(
        self: &Arc<Self>,
        url: String,
        text: String,
        suggestion: SuggestionPattern,
        _attributes: Vec<Attribute>, // TODO remove attributes
    ) -> Arc<ComposerUpdate> {
        let url = Utf16String::from_str(&url);
        let text = Utf16String::from_str(&html_escape::encode_safe(&text));
        let suggestion = wysiwyg::SuggestionPattern::from(suggestion);
        let attrs = vec![];
        Arc::new(ComposerUpdate::from(
            self.inner
                .lock()
                .unwrap()
                .insert_mention_at_suggestion(url, text, suggestion, attrs),
        ))
    }

    pub fn remove_links(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(
            self.inner.lock().unwrap().remove_links(),
        ))
    }

    pub fn indent(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().indent()))
    }

    pub fn unindent(self: &Arc<Self>) -> Arc<ComposerUpdate> {
        Arc::new(ComposerUpdate::from(self.inner.lock().unwrap().unindent()))
    }

    pub fn to_example_format(self: &Arc<Self>) -> String {
        self.inner.lock().unwrap().to_example_format()
    }

    pub fn to_tree(self: &Arc<Self>) -> String {
        self.inner.lock().unwrap().to_tree().to_string()
    }

    pub fn get_current_dom_state(self: &Arc<Self>) -> ComposerState {
        self.inner
            .lock()
            .unwrap()
            .get_current_state()
            .clone()
            .into()
    }

    pub fn action_states(
        self: &Arc<Self>,
    ) -> HashMap<ComposerAction, ActionState> {
        self.inner.lock().unwrap().action_states().into_ffi()
    }

    pub fn get_link_action(self: &Arc<Self>) -> LinkAction {
        self.inner.lock().unwrap().get_link_action().into()
    }

    pub fn get_mentions_state(self: &Arc<Self>) -> MentionsState {
        self.inner.lock().unwrap().get_mentions_state().into()
    }

    /// Force a panic for test purposes
    pub fn debug_panic(self: &Arc<Self>) {
        #[cfg(debug_assertions)]
        panic!("This should only happen in tests.");
    }
}

#[derive(uniffi::Record)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}
