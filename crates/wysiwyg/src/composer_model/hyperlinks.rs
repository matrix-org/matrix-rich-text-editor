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

use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::{ContainerNodeKind::Link, DomNode};
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{DomLocation, Range};
use crate::{ComposerModel, ComposerUpdate, LinkAction, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn get_link_action(&self) -> LinkAction<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        for loc in range.locations {
            if loc.kind == DomNodeKind::Link {
                let node = self.state.dom.lookup_node(&loc.node_handle);
                let link = node.as_container().unwrap().get_link().unwrap();
                return LinkAction::Edit(link);
            }
        }
        if s == e {
            LinkAction::CreateWithText
        } else {
            LinkAction::Create
        }
    }

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
        for leaf in leaves.into_iter().rev() {
            let handle = &leaf.node_handle;
            let parent = self.state.dom.parent_mut(&leaf.node_handle);
            if let Link(_) = parent.kind() {
                parent.set_link(link.clone());
            } else {
                let node = self.state.dom.lookup_node(handle);
                if let DomNode::Text(t) = node {
                    let text = t.data();
                    let before = text[..leaf.start_offset].to_owned();
                    let during =
                        text[leaf.start_offset..leaf.end_offset].to_owned();
                    let after = text[leaf.end_offset..].to_owned();
                    let mut new_nodes = Vec::new();
                    if !before.is_empty() {
                        new_nodes.push(DomNode::new_text(before));
                    }
                    new_nodes.push(DomNode::new_link(
                        link.clone(),
                        vec![DomNode::new_text(during)],
                    ));
                    if !after.is_empty() {
                        new_nodes.push(DomNode::new_text(after));
                    }
                    self.state.dom.replace(handle, new_nodes);
                }
            }
            // TODO: set link should be able to wrap container nodes, unlike formatting
        }
        self.create_update_replace_all()
    }
}
