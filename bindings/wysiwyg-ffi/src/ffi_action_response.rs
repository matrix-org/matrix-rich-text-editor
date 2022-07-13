pub enum ActionResponse {
    Dummy,
}

impl From<ActionResponse> for wysiwyg::ActionResponse {
    fn from(inner: ActionResponse) -> Self {
        match inner {
            ActionResponse::Dummy => wysiwyg::ActionResponse::Dummy,
        }
    }
}
