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
    ComposerModel, ComposerUpdate, DomNode, Location, SuggestionPattern,
    UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Remove the suggestion text and then add a mention to the composer
    pub fn set_mention_from_suggestion(
        &mut self,
        url: S,
        text: S,
        suggestion: SuggestionPattern,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        // This function removes the text between the suggestion start and end points, updates the
        // cursor position and then calls insert_mention (equivalent to link insertion steps)
        self.do_replace_text_in(S::default(), suggestion.start, suggestion.end);
        self.state.start = Location::from(suggestion.start);
        self.state.end = self.state.start;

        self.insert_mention(url, text, attributes)
    }

    /// Inserts a mention at the current selection
    pub fn insert_mention(
        &mut self,
        url: S,
        text: S,
        attributes: Vec<(S, S)>,
    ) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);

        let new_node = DomNode::new_mention(url, text, attributes);
        let new_cursor_index = s + new_node.text_len();

        self.push_state_to_history();

        let handle = self.state.dom.insert_node_at_range(&range, new_node);

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
}
