pub enum TextUpdate {
    Keep,
    ReplaceAll { replacement_text: String },
}

impl TextUpdate {
    pub fn from(inner: wysiwyg::TextUpdate) -> Self {
        match inner {
            wysiwyg::TextUpdate::Keep => Self::Keep,
            wysiwyg::TextUpdate::ReplaceAll(replacement_text) => {
                Self::ReplaceAll { replacement_text }
            }
        }
    }
}
