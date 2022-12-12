use widestring::Utf16String;

pub enum LinkAction {
    Insert,
    Create,
    Edit { link: String, text: String },
}

impl From<wysiwyg::LinkAction<Utf16String>> for LinkAction {
    fn from(inner: wysiwyg::LinkAction<Utf16String>) -> Self {
        match inner {
            wysiwyg::LinkAction::Insert => Self::Insert,
            wysiwyg::LinkAction::Create => Self::Create,
            wysiwyg::LinkAction::Edit { link, text } => Self::Edit {
                link: link.to_string(),
                text: text.to_string(),
            },
        }
    }
}
