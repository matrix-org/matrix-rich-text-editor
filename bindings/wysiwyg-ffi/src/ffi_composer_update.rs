use widestring::Utf16String;

use crate::ffi_menu_state::MenuState;
use crate::ffi_text_update::TextUpdate;
use crate::MenuAction;

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

    pub fn menu_action(&self) -> MenuAction {
        MenuAction::from(self.inner.menu_action.clone())
    }
}

#[cfg(test)]
mod test {
    use std::{collections::HashMap, sync::Arc};

    use crate::{
        ActionState, ComposerAction, ComposerModel, MenuAction, MenuState,
        SuggestionPattern,
    };

    #[test]
    fn initial_menu_update_is_populated() {
        let model = Arc::new(ComposerModel::new());
        let update = model.replace_text(String::from(""));

        // Only Redo is disabled
        assert_eq!(
            update.menu_state(),
            MenuState::Update {
                action_states: redo_indent_unindent_disabled()
            }
        );
    }

    #[test]
    fn after_set_content_from_html_menu_is_updated() {
        let model = Arc::new(ComposerModel::new());
        let update = model.set_content_from_html(String::from("")).unwrap();

        // Undo and Redo are disabled
        assert_eq!(
            update.menu_state(),
            MenuState::Update {
                action_states: undo_redo_indent_unindent_disabled()
            }
        );
    }

    #[test]
    fn after_later_set_content_from_html_menu_is_updated() {
        let model = Arc::new(ComposerModel::new());
        model.replace_text(String::from("foo"));
        model.replace_text(String::from("bar"));
        model.undo();
        let update = model.set_content_from_html(String::from("")).unwrap();

        // Undo and Redo are disabled
        assert_eq!(
            update.menu_state(),
            MenuState::Update {
                action_states: undo_redo_indent_unindent_disabled()
            }
        );
    }

    #[test]
    fn after_set_content_from_markdown_menu_is_updated() {
        let model = Arc::new(ComposerModel::new());
        let update = model.set_content_from_markdown(String::from("")).unwrap();

        // Undo and Redo are disabled
        assert_eq!(
            update.menu_state(),
            MenuState::Update {
                action_states: undo_redo_indent_unindent_disabled()
            }
        );
    }

    #[test]
    fn initial_menu_action_is_none() {
        let model = Arc::new(ComposerModel::new());
        let update = model.set_content_from_html("".into()).unwrap();

        assert_eq!(update.menu_action(), MenuAction::None);
    }

    #[test]
    fn menu_action_is_updated() {
        let model = Arc::new(ComposerModel::new());
        let update = model.replace_text("@alic".into());

        assert_eq!(
            update.menu_action(),
            MenuAction::Suggestion {
                suggestion_pattern: SuggestionPattern {
                    key: crate::PatternKey::At,
                    text: "alic".into(),
                    start: 0,
                    end: 5
                }
            },
        )
    }

    fn redo_indent_unindent_disabled() -> HashMap<ComposerAction, ActionState> {
        HashMap::from([
            (ComposerAction::Bold, ActionState::Enabled),
            (ComposerAction::Indent, ActionState::Disabled),
            (ComposerAction::InlineCode, ActionState::Enabled),
            (ComposerAction::Italic, ActionState::Enabled),
            (ComposerAction::Link, ActionState::Enabled),
            (ComposerAction::OrderedList, ActionState::Enabled),
            (ComposerAction::Redo, ActionState::Disabled),
            (ComposerAction::StrikeThrough, ActionState::Enabled),
            (ComposerAction::Unindent, ActionState::Disabled),
            (ComposerAction::Underline, ActionState::Enabled),
            (ComposerAction::Undo, ActionState::Enabled),
            (ComposerAction::UnorderedList, ActionState::Enabled),
            (ComposerAction::CodeBlock, ActionState::Enabled),
            (ComposerAction::Quote, ActionState::Enabled),
        ])
    }

    fn undo_redo_indent_unindent_disabled(
    ) -> HashMap<ComposerAction, ActionState> {
        HashMap::from([
            (ComposerAction::Bold, ActionState::Enabled),
            (ComposerAction::Indent, ActionState::Disabled),
            (ComposerAction::InlineCode, ActionState::Enabled),
            (ComposerAction::Italic, ActionState::Enabled),
            (ComposerAction::Link, ActionState::Enabled),
            (ComposerAction::OrderedList, ActionState::Enabled),
            (ComposerAction::Redo, ActionState::Disabled),
            (ComposerAction::StrikeThrough, ActionState::Enabled),
            (ComposerAction::Unindent, ActionState::Disabled),
            (ComposerAction::Underline, ActionState::Enabled),
            (ComposerAction::Undo, ActionState::Disabled),
            (ComposerAction::UnorderedList, ActionState::Enabled),
            (ComposerAction::CodeBlock, ActionState::Enabled),
            (ComposerAction::Quote, ActionState::Enabled),
        ])
    }
}
