pub enum TextUpdate {
    Keep,
    ReplaceAll {
        replacement_html: String,
        selection_start_codepoint: u32,
        selection_end_codepoint: u32,
    },
}

impl TextUpdate {
    pub fn from(inner: wysiwyg::TextUpdate) -> Self {
        match inner {
            wysiwyg::TextUpdate::Keep => Self::Keep,
            wysiwyg::TextUpdate::ReplaceAll(replace_all) => Self::ReplaceAll {
                replacement_html: replace_all.replacement_html,
                selection_start_codepoint: u32::try_from(
                    replace_all.selection_start_codepoint,
                )
                .unwrap(),
                selection_end_codepoint: u32::try_from(
                    replace_all.selection_end_codepoint,
                )
                .unwrap(),
            },
        }
    }
}
