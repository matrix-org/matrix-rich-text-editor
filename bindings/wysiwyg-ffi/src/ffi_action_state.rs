#[derive(Debug, PartialEq)]
pub enum ActionState {
    Enabled,
    Reversed,
    Disabled,
}

impl From<&wysiwyg::ActionState> for ActionState {
    fn from(inner: &wysiwyg::ActionState) -> Self {
        match inner {
            wysiwyg::ActionState::Enabled => Self::Enabled,
            wysiwyg::ActionState::Reversed => Self::Reversed,
            wysiwyg::ActionState::Disabled => Self::Disabled,
        }
    }
}
