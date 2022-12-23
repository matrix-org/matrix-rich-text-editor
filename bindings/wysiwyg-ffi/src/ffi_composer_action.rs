#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
    CodeBlock,
    Quote,
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
            ComposerAction::CodeBlock => Self::CodeBlock,
            ComposerAction::Quote => Self::Quote,
        }
    }
}

impl From<&wysiwyg::ComposerAction> for ComposerAction {
    fn from(action: &wysiwyg::ComposerAction) -> Self {
        match action {
            wysiwyg::ComposerAction::Bold => Self::Bold,
            wysiwyg::ComposerAction::Italic => Self::Italic,
            wysiwyg::ComposerAction::StrikeThrough => Self::StrikeThrough,
            wysiwyg::ComposerAction::Underline => Self::Underline,
            wysiwyg::ComposerAction::InlineCode => Self::InlineCode,
            wysiwyg::ComposerAction::Link => Self::Link,
            wysiwyg::ComposerAction::Undo => Self::Undo,
            wysiwyg::ComposerAction::Redo => Self::Redo,
            wysiwyg::ComposerAction::OrderedList => Self::OrderedList,
            wysiwyg::ComposerAction::UnorderedList => Self::UnorderedList,
            wysiwyg::ComposerAction::Indent => Self::Indent,
            wysiwyg::ComposerAction::UnIndent => Self::UnIndent,
            wysiwyg::ComposerAction::CodeBlock => Self::CodeBlock,
            wysiwyg::ComposerAction::Quote => Self::Quote,
        }
    }
}
