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

use crate::composer_model::base::{slice, slice_from, slice_to};
use crate::dom::nodes::DomNode;
use crate::dom::{Range, SameNodeRange};
use crate::{ComposerModel, ComposerUpdate, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn set_link(&mut self, link: S) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        // Can't add a link to an empty selection
        if s == e {
            return ComposerUpdate::keep();
        }
        // Store current Dom
        self.push_state_to_history();

        let range = self.state.dom.find_range(s, e);
        match range {
            Range::SameNode(range) => {
                self.set_link_same_node(range, link);
                // TODO: for now, we replace every time, to check ourselves, but
                // at least some of the time we should not
                return self.create_update_replace_all();
            }

            Range::NoNode => {
                panic!("Can't add link to empty range");
            }

            _ => panic!("Can't add link in complex object models yet"),
        }
    }

    fn set_link_same_node(&mut self, range: SameNodeRange, link: S) {
        // TODO: set link should be able to wrap container nodes, unlike formatting
        let node = self.state.dom.lookup_node(range.node_handle.clone());
        if let DomNode::Text(t) = node {
            let text = t.data();
            let before = slice_to(text, ..range.start_offset);
            let during = slice(text, range.start_offset..range.end_offset);
            let after = slice_from(text, range.end_offset..);
            let new_nodes = vec![
                DomNode::new_text(before),
                DomNode::new_link(link, vec![DomNode::new_text(during)]),
                DomNode::new_text(after),
            ];
            self.state.dom.replace(range.node_handle, new_nodes);
        } else {
            panic!("Trying to bold a non-text node")
        }
    }
}
