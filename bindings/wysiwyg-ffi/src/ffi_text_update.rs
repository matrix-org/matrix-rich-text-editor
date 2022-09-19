use widestring::Utf16String;

pub enum TextUpdate {
    Keep,
    ReplaceAll {
        replacement_html: Vec<u16>,
        start_utf16_codeunit: u32,
        end_utf16_codeunit: u32,
    },
    Select {
        start_utf16_codeunit: u32,
        end_utf16_codeunit: u32,
    },
}

impl TextUpdate {
    pub fn from(inner: wysiwyg::TextUpdate<Utf16String>) -> Self {
        match inner {
            wysiwyg::TextUpdate::Keep => Self::Keep,
            wysiwyg::TextUpdate::ReplaceAll(replace_all) => {
                let start_utf16_codeunit: usize = replace_all.start.into();
                let end_utf16_codeunit: usize = replace_all.end.into();
                Self::ReplaceAll {
                    replacement_html: replace_all.replacement_html.into_vec(),
                    start_utf16_codeunit: u32::try_from(start_utf16_codeunit)
                        .unwrap(),
                    end_utf16_codeunit: u32::try_from(end_utf16_codeunit)
                        .unwrap(),
                }
            }
            wysiwyg::TextUpdate::Select(selection) => {
                let start_utf16_codeunit: usize = selection.start.into();
                let end_utf16_codeunit: usize = selection.end.into();
                Self::Select {
                    start_utf16_codeunit: u32::try_from(start_utf16_codeunit)
                        .unwrap(),
                    end_utf16_codeunit: u32::try_from(end_utf16_codeunit)
                        .unwrap(),
                }
            }
        }
    }
}
