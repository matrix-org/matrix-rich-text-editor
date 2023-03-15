use widestring::Utf16String;

pub enum LinkAction {
    CreateWithText,
    Create,
    Edit { url: String, text: String },
}

impl From<wysiwyg::LinkAction<Utf16String>> for LinkAction {
    fn from(inner: wysiwyg::LinkAction<Utf16String>) -> Self {
        match inner {
            wysiwyg::LinkAction::CreateWithText => Self::CreateWithText,
            wysiwyg::LinkAction::Create => Self::Create,
            wysiwyg::LinkAction::Edit { url, text } => Self::Edit {
                url: url.to_string(),
                text: text.to_string(),
            },
        }
    }
}
