pub enum InlineFormatType {
    Bold,
    Italic,
    StrikeThrough,
    Underline,
}

impl From<InlineFormatType> for wysiwyg::InlineFormatType {
    fn from(inner: InlineFormatType) -> Self {
        match inner {
            InlineFormatType::Bold => wysiwyg::InlineFormatType::Bold,
            InlineFormatType::Italic => wysiwyg::InlineFormatType::Italic,
            InlineFormatType::StrikeThrough => {
                wysiwyg::InlineFormatType::StrikeThrough
            }
            InlineFormatType::Underline => wysiwyg::InlineFormatType::Underline,
        }
    }
}
