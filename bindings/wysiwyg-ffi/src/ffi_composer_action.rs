use strum_macros::{AsRefStr, EnumIter};

#[derive(AsRefStr, Clone, Copy, EnumIter, Eq, Hash, PartialEq)]
pub enum ComposerAction {
    Bold,
    Italic,
    StrikeThrough,
    Underline,
    InlineCode,
    Link,
    Undo,
    Redo,
    OrderedList,
    UnorderedList,
    Indent,
    UnIndent,
}

impl From<&ComposerAction> for wysiwyg::ComposerAction {
    fn from(action: &ComposerAction) -> Self {
        match action {
            ComposerAction::Bold => Self::Bold,
            ComposerAction::Italic => Self::Italic,
            ComposerAction::StrikeThrough => Self::StrikeThrough,
            ComposerAction::Underline => Self::Underline,
            ComposerAction::InlineCode => Self::InlineCode,
            ComposerAction::Link => Self::Link,
            ComposerAction::Undo => Self::Undo,
            ComposerAction::Redo => Self::Redo,
            ComposerAction::OrderedList => Self::OrderedList,
            ComposerAction::UnorderedList => Self::UnorderedList,
            ComposerAction::Indent => Self::Indent,
            ComposerAction::UnIndent => Self::UnIndent,
        }
    }
}
