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

use ruma_common::{
    matrix_uri::MatrixId, IdParseError, MatrixToUri, MatrixUri, UserId,
};

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
    Room,
    User,
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
        let text = match parse_matrix_id(room_uri)? {
            MatrixId::Room(room_id) => room_id.to_string(),
            MatrixId::RoomAlias(room_alias) => room_alias.to_string(),
            _ => return None,
        };

        Some(Mention::new(
            room_uri.to_string(),
            text.clone(),
            text,
            MentionKind::Room,
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
        Some(matrix_uri.id().to_owned())
    } else if let Ok(matrix_to_uri) = MatrixToUri::parse(uri) {
        Some(matrix_to_uri.id().to_owned())
    } else if let Ok(matrix_to_uri) = parse_external_id(uri) {
        Some(matrix_to_uri.id().to_owned())
    } else {
        None
    }
}

/// Attempts to split an external id on `/#/`, rebuild as a matrix to style permalink then parse
/// using ruma.
///
/// Returns the result of calling `parse` in ruma.
fn parse_external_id(uri: &str) -> Result<MatrixToUri, IdParseError> {
    // first split the string into the parts we need
    let parts: Vec<&str> = uri.split("/#/").collect();

    // we expect this to split the uri into exactly two parts, if it's anything else, return early
    if parts.len() != 2 {
        return Err(IdParseError::Empty);
    }
    let after_hash = parts[1];

    // now rebuild the string as if it were a matrix to type link, then use ruma to parse
    let uri_for_ruma = format!("{}{}", MATRIX_TO_BASE_URL, after_hash);

    MatrixToUri::parse(&uri_for_ruma)
}

#[cfg(test)]
mod test {
    use ruma_common::{MatrixToUri, MatrixUri};

    use crate::mention::{Mention, MentionKind};

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
        assert_eq!(parsed.kind(), &MentionKind::Room);
    }

    #[test]
    fn parse_uri_matrix_uri_valid_room() {
        let uri = "matrix:roomid/roomid:example.org";
        let parsed = Mention::from_uri(matrix_uri(uri)).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "!roomid:example.org");
        assert_eq!(parsed.display_text(), "!roomid:example.org");
        assert_eq!(parsed.kind(), &MentionKind::Room);
    }

    #[test]
    fn parse_uri_matrix_to_valid_room_alias() {
        let uri = "https://matrix.to/#/#room:example.org";
        let parsed = Mention::from_uri(matrix_to(uri)).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "#room:example.org");
        assert_eq!(parsed.display_text(), "#room:example.org");
        assert_eq!(parsed.kind(), &MentionKind::Room);
    }

    #[test]
    fn parse_uri_matrix_uri_valid_room_alias() {
        let uri = "matrix:r/room:example.org";
        let parsed = Mention::from_uri(matrix_uri(uri)).unwrap();

        assert_eq!(parsed.uri(), uri);
        assert_eq!(parsed.mx_id(), "#room:example.org");
        assert_eq!(parsed.display_text(), "#room:example.org");
        assert_eq!(parsed.kind(), &MentionKind::Room);
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
        assert_eq!(parsed.kind(), &MentionKind::Room);
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
        assert_eq!(parsed.display_text(), "!room:example.org");
        assert_eq!(parsed.kind(), &MentionKind::Room);
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
        assert_eq!(parsed.kind(), &MentionKind::Room);
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
