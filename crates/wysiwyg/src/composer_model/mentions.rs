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
use matrix_mentions::Mention;

use crate::{
    dom::DomLocation, ComposerModel, ComposerUpdate, DomNode, Location,
    SuggestionPattern, UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Remove the suggestion text and then insert a mention into the composer, using the following rules
    /// - Do not insert a mention if the uri is invalid
    /// - Do not insert a mention if the range includes link or code leaves
    /// - If the composer contains a selection, remove the contents of the selection
    /// prior to inserting a mention at the cursor.
    /// - If the composer contains a cursor, insert a mention at the cursor
    pub fn insert_mention_at_suggestion(
        &mut self,
        url: S,
        text: S,
        suggestion: SuggestionPattern,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        if self.should_not_insert_mention(&url, &text) {
            return ComposerUpdate::keep();
        }

        self.push_state_to_history();
        self.do_replace_text_in(S::default(), suggestion.start, suggestion.end);
        self.state.start = Location::from(suggestion.start);
        self.state.end = self.state.start;
        self.do_insert_mention(url, text, attributes)
    }

    /// Inserts a mention into the composer. It uses the following rules:
    /// - Do not insert a mention if the uri is invalid
    /// - Do not insert a mention if the range includes link or code leaves
    /// - If the composer contains a selection, remove the contents of the selection
    /// prior to inserting a mention at the cursor.
    /// - If the composer contains a cursor, insert a mention at the cursor
    pub fn insert_mention(
        &mut self,
        url: S,
        text: S,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        if self.should_not_insert_mention(&url, &text) {
            return ComposerUpdate::keep();
        }

        self.push_state_to_history();
        if self.has_selection() {
            self.do_replace_text(S::default());
        }
        self.do_insert_mention(url, text, attributes)
    }

    /// Creates a new mention node then inserts the node at the cursor position. If creation fails due to
    /// an invalid uri, it will return `ComposerUpdate::keep()`.
    /// It adds a trailing space when the inserted mention is the last node in it's parent.
    fn do_insert_mention(
        &mut self,
        url: S,
        text: S,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        let (start, end) = self.safe_selection();
        let range = self.state.dom.find_range(start, end);

        if let Ok(new_node) = DomNode::new_mention(url, text, attributes) {
            let new_cursor_index = start + new_node.text_len();

            let handle = self.state.dom.insert_node_at_cursor(&range, new_node);

            // manually move the cursor to the end of the mention
            self.state.start = Location::from(new_cursor_index);
            self.state.end = self.state.start;

            // add a trailing space in cases when we do not have a next sibling
            if self.state.dom.is_last_in_parent(&handle) {
                self.do_replace_text(" ".into())
            } else {
                self.create_update_replace_all()
            }
        } else {
            ComposerUpdate::keep()
        }
    }

    /// Utility function for the insert_mention* methods. It returns false if:
    /// - the range includes any link or code type leaves
    /// - the url is not a valid matrix uri (with special case for at-room)
    ///
    /// Related issue is here:
    /// https://github.com/matrix-org/matrix-rich-text-editor/issues/702
    /// We do not allow mentions to be inserted into links, the planned behaviour is
    /// detailed in the above issue.
    fn should_not_insert_mention(&self, url: &S, text: &S) -> bool {
        let (start, end) = self.safe_selection();
        let range = self.state.dom.find_range(start, end);

        let invalid_uri = !Mention::is_valid_uri(url.to_string().as_str());

        let range_contains_link_or_code_leaves =
            range.locations.iter().any(|l: &DomLocation| {
                l.kind.is_link_kind() || l.kind.is_code_kind()
            });

        // when we have an at-room mention, it doesn't matter about the url as we do not use
        // it, rendering the mention as raw text in the html output
        if text == &S::from("@room") {
            range_contains_link_or_code_leaves
        } else {
            invalid_uri || range_contains_link_or_code_leaves
        }
    }
}
