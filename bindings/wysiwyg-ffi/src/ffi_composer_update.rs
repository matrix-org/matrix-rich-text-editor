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
        ActionState, Attribute, ComposerAction, ComposerModel, MenuAction,
        MenuState, SuggestionPattern,
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
                    end: 5,
                }
            },
        )
    }

    #[test]
    fn test_replace_whole_suggestion_with_mention_ffi() {
        let model = Arc::new(ComposerModel::new());
        let update = model.replace_text("@alic".into());

        let MenuAction::Suggestion { suggestion_pattern } =
            update.menu_action() else
        {
            panic!("No suggestion found");
        };

        model.set_link_suggestion(
            "https://matrix.to/#/@alice:matrix.org".into(),
            "Alice".into(),
            suggestion_pattern,
            vec![
                Attribute {
                    key: "contenteditable".into(),
                    value: "false".into(),
                },
                Attribute {
                    key: "data-mention-type".into(),
                    value: "user".into(),
                },
            ],
        );
        assert_eq!(
            model.get_content_as_html(),
            "<a contenteditable=\"false\" data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\">Alice</a>\u{a0}",
        )
    }

    #[test]
    fn test_replace_end_of_text_node_with_mention_ffi() {
        let model = Arc::new(ComposerModel::new());
        model.replace_text("hello ".into());

        let update = model.replace_text("@ali".into());

        let MenuAction::Suggestion { suggestion_pattern } =
            update.menu_action() else
        {
            panic!("No suggestion found");
        };

        model.set_link_suggestion(
            "https://matrix.to/#/@alice:matrix.org".into(),
            "Alice".into(),
            suggestion_pattern,
            vec![Attribute {
                key: "data-mention-type".into(),
                value: "user".into(),
            }],
        );
        assert_eq!(
            model.get_content_as_html(),
            "hello <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>\u{a0}",
        )
    }

    #[test]
    fn test_replace_start_of_text_node_with_mention_ffi() {
        let model = Arc::new(ComposerModel::new());
        model.replace_text(" says hello".into());
        model.select(0, 0);

        let update = model.replace_text("@ali".into());

        let MenuAction::Suggestion { suggestion_pattern } =
            update.menu_action() else
        {
            panic!("No suggestion found");
        };

        model.set_link_suggestion(
            "https://matrix.to/#/@alice:matrix.org".into(),
            "Alice".into(),
            suggestion_pattern,
            vec![Attribute {
                key: "data-mention-type".into(),
                value: "user".into(),
            }],
        );
        assert_eq!(
            model.get_content_as_html(),
            "<a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a> says hello",
        )
    }

    #[test]
    fn test_replace_text_in_middle_of_node_with_mention_ffi() {
        let model = Arc::new(ComposerModel::new());
        model.replace_text("Like  said".into());
        model.select(5, 5); // "Like | said"

        let update = model.replace_text("@ali".into());

        let MenuAction::Suggestion { suggestion_pattern } =
            update.menu_action() else
        {
            panic!("No suggestion found");
        };

        model.set_link_suggestion(
            "https://matrix.to/#/@alice:matrix.org".into(),
            "Alice".into(),
            suggestion_pattern,
            vec![Attribute {
                key: "data-mention-type".into(),
                value: "user".into(),
            }],
        );
        assert_eq!(
            model.get_content_as_html(),
            "Like <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a> said",
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
