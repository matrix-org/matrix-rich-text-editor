use widestring::Utf16String;

#[derive(uniffi::Enum)]
pub enum LinkAction {
    CreateWithText,
    Create,
    Edit { url: String },
    Disabled,
}

impl From<wysiwyg::LinkAction<Utf16String>> for LinkAction {
    fn from(inner: wysiwyg::LinkAction<Utf16String>) -> Self {
        match inner {
            wysiwyg::LinkAction::CreateWithText => Self::CreateWithText,
            wysiwyg::LinkAction::Create => Self::Create,
            wysiwyg::LinkAction::Edit(url) => Self::Edit {
                url: url.to_string(),
            },
            wysiwyg::LinkAction::Disabled => Self::Disabled,
        }
    }
}

#[derive(uniffi::Enum)]
pub enum LinkActionUpdate {
    Keep,
    Update { link_action: LinkAction },
}

impl From<wysiwyg::LinkActionUpdate<Utf16String>> for LinkActionUpdate {
    fn from(inner: wysiwyg::LinkActionUpdate<Utf16String>) -> Self {
        match inner {
            wysiwyg::LinkActionUpdate::Keep => Self::Keep,
            wysiwyg::LinkActionUpdate::Update(action) => Self::Update {
                link_action: action.into(),
            },
        }
    }
}
