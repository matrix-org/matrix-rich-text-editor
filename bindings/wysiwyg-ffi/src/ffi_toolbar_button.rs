pub enum ToolbarButton {
    OrderedList,
    UnorderedList,
}

impl ToolbarButton {
    pub fn from(inner: wysiwyg::ToolbarButton) -> Self {
        match inner {
            wysiwyg::ToolbarButton::OrderedList => Self::OrderedList,
            wysiwyg::ToolbarButton::UnorderedList => Self::UnorderedList,
        }
    }
}
