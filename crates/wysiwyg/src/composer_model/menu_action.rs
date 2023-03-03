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
        unicode_string::{UnicodeStr, UnicodeStringExt},
        Range,
    },
    ComposerModel, MenuAction, PatternKey, SuggestionPattern, UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Compute the menu action for current composer model state.
    pub(crate) fn compute_menu_action(&self) -> MenuAction {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        if range.locations.iter().any(|l| l.kind.is_code_kind()) {
            return MenuAction::None;
        }
        let (raw_text, start, end) = self.extended_text(range);
        if let Some((key, text)) = Self::pattern_for_text(raw_text, start) {
            MenuAction::Suggestion(SuggestionPattern {
                key,
                text,
                start,
                end,
            })
        } else {
            MenuAction::None
        }
    }

    /// Compute extended text from a range. Text is extended up
    /// to the leading/trailing of the text nodes, or up to the
    /// first whitespace found.
    /// Returns the extended text, and its start/end locations.
    fn extended_text(&self, range: Range) -> (S, usize, usize) {
        range
            .leaves()
            .filter_map(|loc| {
                self.state
                    .dom
                    .lookup_node(&loc.node_handle)
                    .as_text()
                    .map(|t| (t, loc.start_offset..loc.end_offset))
            })
            .fold(
                (S::default(), range.start(), range.end()),
                |(mut text, s, e), (t, range)| {
                    let (node_text, start_offset, end_offset) =
                        t.extended_text_for_range(range);
                    text.push(node_text);
                    (text, s - start_offset, e + end_offset)
                },
            )
    }

    /// Compute at/hash/slash pattern for a given text.
    /// Return pattern key and associated text, if it exists.
    fn pattern_for_text(
        mut text: S,
        start_location: usize,
    ) -> Option<(PatternKey, String)> {
        let Some(first_char) = text.pop_first() else {
            return None;
        };
        let Some(key) = PatternKey::from_char(first_char) else {
            return None;
        };

        // Exclude slash patterns that are not at the beginning of the document
        // and any selection that contains inner whitespaces.
        if (key == PatternKey::Slash && start_location > 0)
            || text.chars().any(|c| c.is_whitespace())
        {
            None
        } else {
            Some((key, text.to_string()))
        }
    }
}
