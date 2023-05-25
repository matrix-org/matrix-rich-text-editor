// Copyright 2022 The Matrix.org Foundation C.I.C.
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

use ruma_common::{matrix_uri::MatrixId, MatrixToUri, MatrixUri};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mention {
    uri: String,
    text: String,
}

impl Mention {
    fn new(uri: String, text: String) -> Self {
        Mention { uri, text }
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn text(&self) -> &str {
        &self.text
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

    /// Create a mention from a link
    ///
    /// If the URI is a valid room, it constructs a room mention, ignoring the
    /// provided `anchor_text` and using the room Itext
    ///
    /// If the URI is a valid user, it constructs a valid room mention, and
    /// assumes the provided `anchor_text` is the user's display name.
    pub fn from_link(href: &str, anchor_text: &str) -> Option<Mention> {
        match parse_matrix_id(href)? {
            MatrixId::Room(_) | MatrixId::RoomAlias(_) => {
                Mention::from_room(href)
            }
            MatrixId::User(_) => Mention::from_user(href, Some(anchor_text)),
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

                Some(Mention::new(user_uri.to_string(), text.to_string()))
            }
            _ => None,
        }
    }

    /// Create a mention from a room URI and an optional display name
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

        Some(Mention::new(room_uri.to_string(), text))
    }
}

fn parse_matrix_id(uri: &str) -> Option<MatrixId> {
    if let Ok(matrix_uri) = MatrixUri::parse(uri) {
        Some(matrix_uri.id().to_owned())
    } else if let Ok(matrix_to_uri) = MatrixToUri::parse(uri) {
        Some(matrix_to_uri.id().to_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use ruma_common::{MatrixToUri, MatrixUri};

    use crate::mention::Mention;

    #[test]
    fn parse_uri_matrix_to_valid_user() {
        let parsed = Mention::from_uri(matrix_to(
            "https://matrix.to/#/@alice:example.org",
        ))
        .unwrap();
        assert!(parsed.text() == "@alice:example.org");
    }

    #[test]
    fn parse_uri_matrix_uri_valid_user() {
        let parsed =
            Mention::from_uri(matrix_uri("matrix:u/alice:example.org"))
                .unwrap();
        assert!(parsed.text() == "@alice:example.org");
    }

    #[test]
    fn parse_uri_matrix_to_valid_room() {
        let parsed = Mention::from_uri(matrix_to(
            "https://matrix.to/#/!roomid:example.org",
        ))
        .unwrap();
        assert!(parsed.text() == "!roomid:example.org");
    }

    #[test]
    fn parse_uri_matrix_uri_valid_room() {
        let parsed =
            Mention::from_uri(matrix_uri("matrix:roomid/roomid:example.org"))
                .unwrap();
        assert!(parsed.text() == "!roomid:example.org");
    }

    #[test]
    fn parse_uri_matrix_to_valid_room_alias() {
        let parsed = Mention::from_uri(matrix_to(
            "https://matrix.to/#/#room:example.org",
        ))
        .unwrap();
        assert!(parsed.text() == "#room:example.org");
    }

    #[test]
    fn parse_uri_matrix_uri_valid_room_alias() {
        let parsed =
            Mention::from_uri(matrix_uri("matrix:r/room:example.org")).unwrap();
        assert!(parsed.text() == "#room:example.org");
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
    fn parse_link_user_text() {
        let parsed = Mention::from_link(
            matrix_to("https://matrix.to/#/@alice:example.org"),
            "Alice",
        )
        .unwrap();
        assert!(parsed.text() == "Alice");
    }

    #[test]
    fn parse_link_room_text() {
        let parsed = Mention::from_link(
            matrix_to("https://matrix.to/#/!room:example.org"),
            "My room",
        )
        .unwrap();
        assert!(parsed.text() == "!room:example.org");
    }

    #[test]
    fn parse_link_room_alias_text() {
        let parsed = Mention::from_link(
            matrix_to("https://matrix.to/#/#room:example.org"),
            "My room",
        )
        .unwrap();
        assert!(parsed.text() == "#room:example.org");
    }

    #[test]
    fn parse_link_event_text() {
        let parsed = Mention::from_link(
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
