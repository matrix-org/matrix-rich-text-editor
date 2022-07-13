pub enum MenuState {
    None,
}

impl MenuState {
    pub fn from(inner: wysiwyg::MenuState) -> Self {
        match inner {
            wysiwyg::MenuState::None => Self::None,
        }
    }
}
