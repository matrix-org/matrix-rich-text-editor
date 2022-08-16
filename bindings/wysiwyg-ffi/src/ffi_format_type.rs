pub enum InlineFormatType {
    Bold,
    Italic,
    Strikethrough,
    Underline,
}

impl From<InlineFormatType> for wysiwyg::InlineFormatType {
    fn from(inner: InlineFormatType) -> Self {
        match inner {
            InlineFormatType::Bold => wysiwyg::InlineFormatType::Bold,
            InlineFormatType::Italic => wysiwyg::InlineFormatType::Italic,
            InlineFormatType::Strikethrough => wysiwyg::InlineFormatType::Strikethrough,
            InlineFormatType::Underline => wysiwyg::InlineFormatType::Underline,
        }
    }
}