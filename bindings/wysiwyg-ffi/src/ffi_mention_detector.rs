use std::sync::Arc;

#[derive(Default, uniffi::Object)]
pub struct MentionDetector {}

impl MentionDetector {
    pub fn new() -> Self {
        Self {}
    }
}

#[uniffi::export]
impl MentionDetector {
    pub fn is_mention(self: &Arc<Self>, url: String) -> bool {
        matrix_mentions::is_mention(&url)
    }

    pub fn get_mention_kind(self: &Arc<Self>, url: String, text: String) -> Option<MentionKind> {
        if let Some(mention) = matrix_mentions::Mention::from_uri_with_display_text(&url, &text) {
            match mention.kind() {
                matrix_mentions::MentionKind::User => Some(MentionKind::User),
                matrix_mentions::MentionKind::Room(id_type) => {
                    let id_type = match id_type {
                        matrix_mentions::RoomIdentificationType::Alias => RoomIdentificationType::Alias,
                        matrix_mentions::RoomIdentificationType::Id => RoomIdentificationType::Id,
                    };
                    Some(MentionKind::Room { id_type })
                }
            }
        } else if text == "@room" || text == "room" {
            Some(MentionKind::Everyone)
        } else {
            None
        }
    }
}

#[derive(uniffi::Enum)]
pub enum MentionKind {
    User,
    Room {
        id_type: RoomIdentificationType
    },
    Everyone,
}

#[derive(uniffi::Enum)]
pub enum RoomIdentificationType {
    Id,
    Alias
}