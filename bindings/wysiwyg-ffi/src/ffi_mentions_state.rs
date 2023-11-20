#[derive(uniffi::Record)]
pub struct MentionsState {
    pub user_ids: Vec<String>,
    pub room_ids: Vec<String>,
    pub room_aliases: Vec<String>,
    pub has_at_room_mention: bool,
}

impl From<wysiwyg::MentionsState> for MentionsState {
    fn from(value: wysiwyg::MentionsState) -> Self {
        Self {
            user_ids: value.user_ids.into_iter().collect(),
            room_ids: value.room_ids.into_iter().collect(),
            room_aliases: value.room_aliases.into_iter().collect(),
            has_at_room_mention: value.has_at_room_mention,
        }
    }
}
