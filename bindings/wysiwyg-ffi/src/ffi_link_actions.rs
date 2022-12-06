use widestring::Utf16String;

pub enum LinkAction {
    CreateWithText,
    Create,
    Edit { link: Vec<u16> },
}

impl From<wysiwyg::LinkAction<Utf16String>> for LinkAction {
    fn from(inner: wysiwyg::LinkAction<Utf16String>) -> Self {
        match inner {
            wysiwyg::LinkAction::CreateWithText => Self::CreateWithText,
            wysiwyg::LinkAction::Create => Self::Create,
            wysiwyg::LinkAction::Edit(link) => Self::Edit {
                link: link.into_vec(),
            },
        }
    }
}
