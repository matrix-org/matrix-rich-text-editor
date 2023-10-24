use std::sync::Arc;
use widestring::Utf16String;

#[derive(Default, uniffi::Object)]
pub struct MentionDetector {}

impl MentionDetector {
    pub fn new() -> Self {
        Self {}
    }
}

#[uniffi::export]
impl MentionDetector {
    pub fn is_user_mention(self: &Arc<Self>, url: String) -> bool {
        let url = Utf16String::from_str(&url);
        wysiwyg::is_user_mention(&url)
    }
}
