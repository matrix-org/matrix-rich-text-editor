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
        nodes::dom_node::DomNodeKind,
        unicode_string::{UnicodeStr, UnicodeStringExt},
        DomLocation,
    },
    ComposerModel, DomNode, MenuAction, PatternKey, SuggestionPattern,
    UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub(crate) fn compute_menu_action(&self) -> MenuAction {
        let (mut s, mut e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);

        if range.locations.iter().any(|l| l.kind.is_code_kind()) {
            return MenuAction::None;
        }

        let text_nodes_loc: Vec<&DomLocation> = range
            .leaves()
            .filter(|l| l.kind == DomNodeKind::Text)
            .collect();

        let mut raw_text = S::default();
        for loc in text_nodes_loc {
            if let DomNode::Text(t) =
                self.state.dom.lookup_node(&loc.node_handle)
            {
                let (text, extended_start, extended_end) =
                    t.extended_text_for_range(loc.start_offset..loc.end_offset);
                raw_text.push(text);
                s -= extended_start;
                e += extended_end;
            }
        }

        if raw_text != S::default() {
            let first_char = raw_text.remove_at(0);
            if let Some(pattern_key) = PatternKey::from_char(first_char) {
                // Pattern found, verify there is no whitespace inside
                if raw_text
                    .chars()
                    .any(|c| matches!(&c, ' ' | '\x09'..='\x0d'))
                {
                    MenuAction::None
                } else {
                    let suggestion_pattern = SuggestionPattern {
                        key: pattern_key,
                        text: raw_text.to_string(),
                        start: s,
                        end: e,
                    };
                    MenuAction::Suggestion(suggestion_pattern)
                }
            } else {
                MenuAction::None
            }
        } else {
            MenuAction::None
        }
    }
}
