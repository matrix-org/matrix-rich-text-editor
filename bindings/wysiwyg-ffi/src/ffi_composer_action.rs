use crate::ActionRequest;

pub struct ComposerAction {
    inner: wysiwyg::ComposerAction,
}

impl ComposerAction {
    pub fn from(inner: wysiwyg::ComposerAction) -> Self {
        Self { inner }
    }

    pub fn action_id(&self) -> String {
        self.inner.action_id.clone()
    }

    pub fn action(&self) -> ActionRequest {
        ActionRequest::from(self.inner.action.clone())
    }
}
