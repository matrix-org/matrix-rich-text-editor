use widestring::Utf16String;

#[derive(uniffi::Enum)]
pub enum LinkAction {
    Keep,
    CreateWithText,
    Create,
    Edit { url: String },
    Disabled,
}

impl From<wysiwyg::LinkAction<Utf16String>> for LinkAction {
    fn from(inner: wysiwyg::LinkAction<Utf16String>) -> Self {
        match inner {
            wysiwyg::LinkAction::Keep => Self::Keep,
            wysiwyg::LinkAction::CreateWithText => Self::CreateWithText,
            wysiwyg::LinkAction::Create => Self::Create,
            wysiwyg::LinkAction::Edit(url) => Self::Edit {
                url: url.to_string(),
            },
            wysiwyg::LinkAction::Disabled => Self::Disabled,
        }
    }
}
