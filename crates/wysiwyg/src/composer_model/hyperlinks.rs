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

use crate::dom::nodes::DomNode;
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{DomLocation, Range};
use crate::{ComposerModel, ComposerUpdate, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn set_link(&mut self, link: S) -> ComposerUpdate<S> {
        // push_state_to_history is after this check:
        let (s, e) = self.safe_selection();
        // Can't add a link to an empty selection
        if s == e {
            return ComposerUpdate::keep();
        }
        self.push_state_to_history();

        let range = self.state.dom.find_range(s, e);
        self.set_link_range(range, link)
    }

    fn set_link_range(&mut self, range: Range, link: S) -> ComposerUpdate<S> {
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        if leaves.len() == 1 {
            let location = leaves[0];
            let handle = &location.node_handle;

            // TODO: set link should be able to wrap container nodes, unlike formatting
            let node = self.state.dom.lookup_node(handle);
            if let DomNode::Text(t) = node {
                let text = t.data();
                let before = text[..location.start_offset].to_owned();
                let during =
                    text[location.start_offset..location.end_offset].to_owned();
                let after = text[location.end_offset..].to_owned();
                let mut new_nodes = Vec::new();
                if !before.is_empty() {
                    new_nodes.push(DomNode::new_text(before));
                }
                new_nodes.push(DomNode::new_link(
                    link,
                    vec![DomNode::new_text(during)],
                ));
                if !after.is_empty() {
                    new_nodes.push(DomNode::new_text(after));
                }
                self.state.dom.replace(handle, new_nodes);
                self.create_update_replace_all()
            } else {
                panic!("Trying to linkify a non-text node")
            }
        } else {
            panic!("Can't add link in complex object models yet")
        }
    }
}
