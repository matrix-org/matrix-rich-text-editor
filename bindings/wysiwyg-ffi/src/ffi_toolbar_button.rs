pub enum ToolbarButton {
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
}

impl ToolbarButton {
    pub fn from(inner: wysiwyg::ToolbarButton) -> Self {
        match inner {
            wysiwyg::ToolbarButton::Bold => Self::Bold,
            wysiwyg::ToolbarButton::Italic => Self::Italic,
            wysiwyg::ToolbarButton::StrikeThrough => Self::StrikeThrough,
            wysiwyg::ToolbarButton::Underline => Self::Underline,
            wysiwyg::ToolbarButton::InlineCode => Self::InlineCode,
            wysiwyg::ToolbarButton::Link => Self::Link,
            wysiwyg::ToolbarButton::Undo => Self::Undo,
            wysiwyg::ToolbarButton::Redo => Self::Redo,
            wysiwyg::ToolbarButton::OrderedList => Self::OrderedList,
            wysiwyg::ToolbarButton::UnorderedList => Self::UnorderedList,
        }
    }
}
