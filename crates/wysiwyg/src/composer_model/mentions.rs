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

use crate::{
    dom::{
        nodes::{MentionNode, MentionNodeKind},
        DomLocation,
    },
    ComposerModel, ComposerUpdate, DomNode, Location, MentionsState,
    SuggestionPattern, UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Returns the current mentions state of the content of the RTE editor.
    pub fn get_mentions_state(&self) -> MentionsState {
        let mut mentions_state = MentionsState::default();
        for node in self.state.dom.iter_mentions() {
            match node.kind() {
                MentionNodeKind::AtRoom => {
                    mentions_state.has_at_room_mention = true
                }
                MentionNodeKind::MatrixUri { mention } => match mention.kind() {
                    matrix_mentions::MentionKind::Room(id_type) => {
                        match id_type {
                            matrix_mentions::RoomIdentificationType::Id => {
                                mentions_state
                                    .room_ids
                                    .insert(mention.mx_id().to_string());
                            }
                            matrix_mentions::RoomIdentificationType::Alias => {
                                mentions_state
                                    .room_aliases
                                    .insert(mention.mx_id().to_string());
                            }
                        }
                    }
                    matrix_mentions::MentionKind::User => {
                        mentions_state
                            .user_ids
                            .insert(mention.mx_id().to_string());
                    }
                },
            }
        }
        mentions_state
    }

    /// Checks to see if the mention should be inserted and also if the mention can be created.
    /// If both of these checks are passed it will remove the suggestion and then insert a mention.
    pub fn insert_mention_at_suggestion(
        &mut self,
        url: S,
        text: S,
        suggestion: SuggestionPattern,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        if self.range_contains_link_or_code_leaves() {
            return ComposerUpdate::keep();
        }

        if let Ok(mention_node) = DomNode::new_mention(url, text, attributes) {
            self.push_state_to_history();
            self.do_replace_text_in(
                S::default(),
                suggestion.start,
                suggestion.end,
            );
            self.state.start = Location::from(suggestion.start);
            self.state.end = self.state.start;
            self.do_insert_mention(mention_node)
        } else {
            ComposerUpdate::keep()
        }
    }

    /// Checks to see if the mention should be inserted and also if the mention can be created.
    /// If both of these checks are passed it will remove any selection if present and then insert a mention.
    pub fn insert_mention(
        &mut self,
        url: S,
        text: S,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        if self.range_contains_link_or_code_leaves() {
            return ComposerUpdate::keep();
        }

        if let Ok(mention_node) = DomNode::new_mention(url, text, attributes) {
            self.push_state_to_history();
            if self.has_selection() {
                self.do_replace_text(S::default());
            }
            self.do_insert_mention(mention_node)
        } else {
            ComposerUpdate::keep()
        }
    }

    /// Checks to see if the at-room mention should be inserted.
    /// If so it will remove the suggestion and then insert an at-room mention.
    pub fn insert_at_room_mention_at_suggestion(
        &mut self,
        suggestion: SuggestionPattern,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        if self.range_contains_link_or_code_leaves() {
            return ComposerUpdate::keep();
        }

        self.push_state_to_history();
        self.do_replace_text_in(S::default(), suggestion.start, suggestion.end);
        self.state.start = Location::from(suggestion.start);
        self.state.end = self.state.start;

        let mention_node = DomNode::new_at_room_mention(attributes);
        self.do_insert_mention(mention_node)
    }

    /// Checks to see if the at-room mention should be inserted.
    /// If so it will remove any selection if present and then insert an at-room mention.
    pub fn insert_at_room_mention(
        &mut self,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        if self.range_contains_link_or_code_leaves() {
            return ComposerUpdate::keep();
        }

        self.push_state_to_history();
        if self.has_selection() {
            self.do_replace_text(S::default());
        }

        let mention_node = DomNode::new_at_room_mention(attributes);
        self.do_insert_mention(mention_node)
    }

    /// Inserts the node at the cursor position. It adds a trailing space when the inserted
    /// mention is the last node in it's parent.
    fn do_insert_mention(
        &mut self,
        mention_node: MentionNode<S>,
    ) -> ComposerUpdate<S> {
        let (start, end) = self.safe_selection();
        let range = self.state.dom.find_range(start, end);

        let new_cursor_index = start + mention_node.text_len();

        let handle = self
            .state
            .dom
            .insert_node_at_cursor(&range, DomNode::Mention(mention_node));

        // manually move the cursor to the end of the mention
        self.state.start = Location::from(new_cursor_index);
        self.state.end = self.state.start;

        // add a trailing space in cases when we do not have a next sibling
        if self.state.dom.is_last_in_parent(&handle) {
            self.do_replace_text(" ".into())
        } else {
            self.create_update_replace_all()
        }
    }

    /// We should not insert a mention if the uri is invalid or the range contains link
    /// or code leaves. See issue https://github.com/matrix-org/matrix-rich-text-editor/issues/702.
    fn range_contains_link_or_code_leaves(&self) -> bool {
        let (start, end) = self.safe_selection();
        let range = self.state.dom.find_range(start, end);

        range.locations.iter().any(|l: &DomLocation| {
            l.kind.is_link_kind() || l.kind.is_code_kind()
        })
    }
}
