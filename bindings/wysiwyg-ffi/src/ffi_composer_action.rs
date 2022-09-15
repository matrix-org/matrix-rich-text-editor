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

impl ComposerAction {
    pub fn from(inner: wysiwyg::ComposerAction) -> Self {
        match inner {
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
        }
    }
}
