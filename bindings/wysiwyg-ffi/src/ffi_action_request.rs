pub enum ActionRequest {
    Dummy,
}

impl ActionRequest {
    pub fn from(inner: wysiwyg::ActionRequest) -> Self {
        match inner {
            wysiwyg::ActionRequest::Dummy => Self::Dummy,
        }
    }
}
