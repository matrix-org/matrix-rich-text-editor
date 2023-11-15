// Copyright 2023 The Matrix.org Foundation C.I.C.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use ruma_common::{matrix_uri::MatrixId, IdParseError, MatrixToUri, MatrixUri};

const MATRIX_TO_BASE_URL: &str = "https://matrix.to/#/";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mention {
    uri: String,
    mx_id: String,
    display_text: String,
    kind: MentionKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MentionKind {
    Room(RoomIdentificationType),
    User,
}

impl MentionKind {
    pub fn is_room(&self) -> bool {
        matches!(self, MentionKind::Room(_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RoomIdentificationType {
    Id,
    Alias,
}

impl Mention {
    fn new(
        uri: String,
        mx_id: String,
        display_text: String,
        kind: MentionKind,
    ) -> Self {
        Mention {
            uri,
            mx_id,
            display_text,
            kind,
        }
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn display_text(&self) -> &str {
        &self.display_text
    }

    pub fn mx_id(&self) -> &str {
        &self.mx_id
    }

    pub fn kind(&self) -> &MentionKind {
        &self.kind
    }

    /// Determine if a uri is a valid matrix uri
    pub fn is_valid_uri(uri: &str) -> bool {
        parse_matrix_id(uri).is_some()
    }

    /// Create a mention from a URI
    ///
    /// If the URI is a valid room or user, it creates a mention using the
    /// default text.
    pub fn from_uri(uri: &str) -> Option<Mention> {
        match parse_matrix_id(uri)? {
            MatrixId::Room(_) | MatrixId::RoomAlias(_) => {
                Mention::from_room(uri)
            }
            MatrixId::User(_) => Mention::from_user(uri, None),
            // TODO: handle MatrixId::Event
            _ => None,
        }
    }

    /// Create a mention from a URI with associated display text
    ///
    /// If the URI is a valid room, it constructs a room mention, ignoring the
    /// provided `display_text` and using the room Itext
    ///
    /// If the URI is a valid user, it constructs a valid room mention, and
    /// assumes the provided `display_text` is the user's display name.
    pub fn from_uri_with_display_text(
        uri: &str,
        display_text: &str,
    ) -> Option<Mention> {
        match parse_matrix_id(uri)? {
            MatrixId::Room(_) | MatrixId::RoomAlias(_) => {
                Mention::from_room(uri)
            }
            MatrixId::User(_) => Mention::from_user(uri, Some(display_text)),
            _ => None,
        }
    }

    /// Create a mention from a user URI and an optional display name
    ///
    /// If the URI is not a valid user, it returns None.
    /// If the display name is not given, it falls back to the user's ID.
    fn from_user(
        user_uri: &str,
        display_name: Option<&str>,
    ) -> Option<Mention> {
        match parse_matrix_id(user_uri)? {
            MatrixId::User(user_id) => {
                // Use the user’s potentially ambiguous display name for the
                // anchor’s text.
                let text = display_name
                    // If the user does not have a display name,
                    // use the user’s ID.
                    .unwrap_or(user_id.as_str());

                Some(Mention::new(
                    user_uri.to_string(),
                    user_id.to_string(),
                    text.to_string(),
                    MentionKind::User,
                ))
            }
            _ => None,
        }
    }

    /// Create a mention from a room URI
    ///
    /// If the URI is not a valid room, it returns None.
    fn from_room(room_uri: &str) -> Option<Mention> {
        // In all cases, use the alias/room ID being linked to as the
        // anchor’s text.
        let room_id_type: RoomIdentificationType;
        let text = match parse_matrix_id(room_uri)? {
            MatrixId::Room(room_id) => {
                room_id_type = RoomIdentificationType::Id;
                room_id.to_string()
            }
            MatrixId::RoomAlias(room_alias) => {
                room_id_type = RoomIdentificationType::Alias;
                room_alias.to_string()
            }
            _ => return None,
        };

        Some(Mention::new(
            room_uri.to_string(),
            text.clone(),
            text,
            MentionKind::Room(room_id_type),
        ))
    }
}

/// Determines if a uri can be parsed for a matrix id. Attempts to treat the uri in three
/// ways when parsing:
/// 1 - As a matrix uri
/// 2 - As a matrix to uri
/// 3 - As a custom uri
///  
/// If any of the above succeed, return Some<MatrixIdI. Else return None.
fn parse_matrix_id(uri: &str) -> Option<MatrixId> {
    if let Ok(matrix_uri) = MatrixUri::parse(uri) {
        return Some(matrix_uri.id().to_owned());
    } else if let Ok(matrix_to_uri) = MatrixToUri::parse(uri) {
        return Some(matrix_to_uri.id().to_owned());
    }

    cfg_if::cfg_if! {
        if #[cfg(any(test, feature = "custom-matrix-urls"))] {
             if let Ok(matrix_to_uri) = parse_external_id(uri) {
            return Some(matrix_to_uri.id().to_owned());
        }
        }
    }

    None
}

/// Attempts to split an external id on `/#/` (or `/#/room/` or /#/user/` if this is based on a URL
/// into a client like Element Web) and rebuild as a matrix.to style permalink then parse using
/// ruma.
///
/// Returns the result of calling `parse` in ruma.

#[cfg(any(test, feature = "custom-matrix-urls"))]
fn parse_external_id(uri: &str) -> Result<MatrixToUri, IdParseError> {
    // first split the string into the parts we need
    let parts: Vec<&str> = split_uri_on_prefix(uri);

    // we expect this to split the uri into exactly two parts, if it's anything else, return early
    if parts.len() != 2 {
        return Err(IdParseError::Empty);
    }
    let after_hash = parts[1];

    // now rebuild the string as if it were a matrix to type link, then use ruma to parse
    let uri_for_ruma = format!("{}{}", MATRIX_TO_BASE_URL, after_hash);

    MatrixToUri::parse(&uri_for_ruma)
}

/// Attempt to find `/#/user/` or `/#/room/` in the supplied URI, and split it on one of those if
/// found, meaning it is a URI into a client like Element Web. Otherwise split it on `/#/`,
/// treating it as if it were a matrix.to URI.
fn split_uri_on_prefix(uri: &str) -> Vec<&str> {
    for pattern in &["/#/user/", "/#/room/", "/#/"] {
        let s: Vec<&str> = uri.split(pattern).collect();
        if s.len() == 2 {
            return s;
        }
    }
    vec![uri]
}

#[cfg(test)]
mod test {
    use ruma_common::{MatrixToUri, MatrixUri};

    use crate::mention::{Mention, MentionKind, RoomIdentificationType};

    #[test]
    fn parse_uri_matrix_to_valid_user() {
        let uri = "https://matrix.to/#/@alice:example.org";
        let parsed = Mention::from_uri(matrix_to(uri)).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "@alice:example.org");
        assert_eq!(parsed.display_text(), "@alice:example.org");
        assert_eq!(parsed.kind(), &MentionKind::User);
    }

    #[test]
    fn parse_uri_matrix_uri_valid_user() {
        let uri = "matrix:u/alice:example.org";
        let parsed = Mention::from_uri(matrix_uri(uri)).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "@alice:example.org");
        assert_eq!(parsed.display_text(), "@alice:example.org");
        assert_eq!(parsed.kind(), &MentionKind::User);
    }

    #[test]
    fn parse_uri_matrix_to_valid_room() {
        let uri = "https://matrix.to/#/!roomid:example.org";
        let parsed = Mention::from_uri(matrix_to(uri)).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "!roomid:example.org");
        assert_eq!(parsed.display_text(), "!roomid:example.org");
        assert_eq!(
            parsed.kind(),
            &MentionKind::Room(RoomIdentificationType::Id)
        );
    }

    #[test]
    fn parse_uri_matrix_uri_valid_room() {
        let uri = "matrix:roomid/roomid:example.org";
        let parsed = Mention::from_uri(matrix_uri(uri)).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "!roomid:example.org");
        assert_eq!(parsed.display_text(), "!roomid:example.org");
        assert_eq!(
            parsed.kind(),
            &MentionKind::Room(RoomIdentificationType::Id)
        );
    }

    #[test]
    fn parse_uri_matrix_to_valid_room_alias() {
        let uri = "https://matrix.to/#/#room:example.org";
        let parsed = Mention::from_uri(matrix_to(uri)).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "#room:example.org");
        assert_eq!(parsed.display_text(), "#room:example.org");
        assert_eq!(
            parsed.kind(),
            &MentionKind::Room(RoomIdentificationType::Alias)
        );
    }

    #[test]
    fn parse_uri_matrix_uri_valid_room_alias() {
        let uri = "matrix:r/room:example.org";
        let parsed = Mention::from_uri(matrix_uri(uri)).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "#room:example.org");
        assert_eq!(parsed.display_text(), "#room:example.org");
        assert_eq!(
            parsed.kind(),
            &MentionKind::Room(RoomIdentificationType::Alias)
        );
    }

    #[test]
    fn parse_uri_matrix_to_valid_event() {
        let parsed = Mention::from_uri(matrix_to(
            "https://matrix.to/#/#room:example.org/$eventid",
        ));
        assert!(parsed.is_none());
    }

    #[test]
    fn parse_uri_matrix_uri_valid_event() {
        let parsed = Mention::from_uri(matrix_uri(
            "matrix:r/room:example.org/e/eventid",
        ));
        assert_eq!(parsed, None);
    }

    #[test]
    fn parse_uri_matrix_to_invalid() {
        assert!(Mention::from_uri("https://matrix.to/#/invalid").is_none());
    }

    #[test]
    fn parse_uri_matrix_uri_invalid() {
        assert!(Mention::from_uri("matrix:u/invalid").is_none());
    }

    #[test]
    fn parse_uri_not_uri() {
        assert!(Mention::from_uri("hello").is_none());
    }

    #[test]
    fn parse_uri_external_user() {
        let uri = "https://custom.custom.com/?secretstuff/#/@alice:example.org";
        let parsed = Mention::from_uri(uri).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "@alice:example.org");
        assert_eq!(parsed.display_text(), "@alice:example.org");
        assert_eq!(parsed.kind(), &MentionKind::User);
    }

    #[test]
    fn parse_uri_external_room() {
        let uri =
            "https://custom.custom.com/?secretstuff/#/!roomid:example.org";
        let parsed = Mention::from_uri(uri).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "!roomid:example.org");
        assert_eq!(parsed.display_text(), "!roomid:example.org");
        assert_eq!(
            parsed.kind(),
            &MentionKind::Room(RoomIdentificationType::Id)
        );
    }

    #[test]
    fn parse_uri_external_permalink_user() {
        // See https://github.com/matrix-org/matrix-react-sdk/blob/9564009eba7986f6a982128175aa45e326823794/src/utils/permalinks/ElementPermalinkConstructor.ts#L34
        // - when configured with a permalink_prefix config value, Element Web creates URLs with
        // "room" or "user" in them.
        // TODO: handle MatrixId::Event in parse_external_id . For example, a URL like:
        // "http://foobar.com/#/room/!roomid:matrix.org/$eventid?via=matrix.org";

        let uri =
            "https://custom.custom.com/?secretstuff/#/user/@alice:example.org";
        let parsed = Mention::from_uri(uri).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "@alice:example.org");
        assert_eq!(parsed.display_text(), "@alice:example.org");
        assert_eq!(parsed.kind(), &MentionKind::User);
    }

    #[test]
    fn parse_uri_external_permalink_room() {
        let uri =
            "https://custom.custom.com/?secretstuff/#/room/!roomid:example.org";
        let parsed = Mention::from_uri(uri).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "!roomid:example.org");
        assert_eq!(parsed.display_text(), "!roomid:example.org");
        assert_eq!(
            parsed.kind(),
            &MentionKind::Room(RoomIdentificationType::Id)
        );
    }

    #[test]
    fn parse_uri_external_permalink_room_alias() {
        let uri =
            "https://custom.custom.com/?secretstuff/#/room/#room_name:example.org";
        let parsed = Mention::from_uri(uri).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "#room_name:example.org");
        assert_eq!(parsed.display_text(), "#room_name:example.org");
        assert_eq!(
            parsed.kind(),
            &MentionKind::Room(RoomIdentificationType::Alias)
        );
    }

    #[test]
    fn parse_link_user_text() {
        let uri = "https://matrix.to/#/@alice:example.org";
        let display_text = "Alice";
        let parsed =
            Mention::from_uri_with_display_text(matrix_to(uri), display_text)
                .unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "@alice:example.org");
        assert_eq!(parsed.display_text(), display_text);
        assert_eq!(parsed.kind(), &MentionKind::User);
    }

    #[test]
    fn parse_link_room_text() {
        let uri = "https://matrix.to/#/!room:example.org";
        let display_text = "My room";
        let parsed =
            Mention::from_uri_with_display_text(matrix_to(uri), display_text)
                .unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "!room:example.org");
        assert_eq!(parsed.display_text(), "!room:example.org"); // note the display_text is overridden
        assert_eq!(
            parsed.kind(),
            &MentionKind::Room(RoomIdentificationType::Id)
        );
    }

    #[test]
    fn parse_link_room_alias_text() {
        let uri = "https://matrix.to/#/#room:example.org";
        let display_text = "My room";
        let parsed =
            Mention::from_uri_with_display_text(matrix_to(uri), display_text)
                .unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "#room:example.org");
        assert_eq!(parsed.display_text(), "#room:example.org"); // note the display_text is overridden
        assert_eq!(
            parsed.kind(),
            &MentionKind::Room(RoomIdentificationType::Alias)
        );
    }

    #[test]
    fn parse_link_event_text() {
        let parsed = Mention::from_uri_with_display_text(
            matrix_to("https://matrix.to/#/#room:example.org/$eventid"),
            "My event",
        );
        assert!(parsed.is_none());
    }

    fn matrix_to(uri: &str) -> &str {
        let parsed = MatrixToUri::parse(uri);
        assert!(parsed.is_ok());
        uri
    }

    fn matrix_uri(uri: &str) -> &str {
        let parsed = MatrixUri::parse(uri);
        assert!(parsed.is_ok());
        uri
    }
}
