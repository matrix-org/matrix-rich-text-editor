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
use crate::dom::nodes::dom_node::{
    DomNodeKind::LineBreak, DomNodeKind::Text, DomNodeKind::Zwsp,
};
use crate::dom::nodes::ContainerNodeKind;
use crate::dom::nodes::{ContainerNodeKind::Link, DomNode};
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{DomLocation, Range};
use crate::{
    ComposerModel, ComposerUpdate, DomHandle, LinkAction, UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn get_link_action(&self) -> LinkAction<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        for loc in range.locations.iter() {
            if loc.kind == DomNodeKind::Link {
                let node = self.state.dom.lookup_node(&loc.node_handle);
                let link = node.as_container().unwrap().get_link().unwrap();
                return LinkAction::Edit(link);
            }
        }
        if s == e || self.is_empty_selection(range) {
            LinkAction::CreateWithText
        } else {
            LinkAction::Create
        }
    }

    fn is_empty_selection(&self, range: Range) -> bool {
        for leaf in range.leaves() {
            if leaf.kind == Zwsp || leaf.kind == LineBreak {
                continue;
            } else if leaf.kind == Text {
                let text_node = self
                    .state
                    .dom
                    .lookup_node(&leaf.node_handle)
                    .as_text()
                    .unwrap();
                let selection_range = leaf.start_offset..leaf.end_offset;
                if !text_node.is_blank_in_range(selection_range) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn set_link_with_text(
        &mut self,
        link: S,
        text: S,
    ) -> ComposerUpdate<S> {
        let (s, mut e) = self.safe_selection();
        self.push_state_to_history();
        self.do_replace_text(text.clone());
        e += text.len();
        let range = self.state.dom.find_range(s, e);
        self.set_link_range(range, link)
    }

    pub fn set_link(&mut self, link: S) -> ComposerUpdate<S> {
        self.push_state_to_history();
        let (mut s, mut e) = self.safe_selection();

        let mut range = self.state.dom.find_range(s, e);

        // Find container link that completely covers the range
        if let Some(link) = self.find_closest_ancestor_link(&range) {
            // If found, update the range to the container link bounds
            range = self.state.dom.find_range_by_node(&link);
            (s, e) = (range.start(), range.end());
        }

        if s == e {
            return ComposerUpdate::keep();
        }

        let inserted = self
            .state
            .dom
            .insert_parent(&range, DomNode::new_link(link, vec![]));

        // Ensure no duplication by deleting any links contained within the new link
        self.delete_child_links(&inserted);

        self.create_update_replace_all()
    }

    fn delete_child_links(&mut self, node_handle: &DomHandle) {
        let node = self.state.dom.lookup_node(node_handle);

        for handle in node
            .iter_containers()
            .filter(|n| matches!(n.kind(), ContainerNodeKind::Link(_)))
            .map(|n| n.handle())
            .filter(|h| *h != *node_handle)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            self.state.dom.remove_and_keep_children(&handle);
        }
    }

    fn find_closest_ancestor_link(
        &mut self,
        range: &Range,
    ) -> Option<DomHandle> {
        let mut parent_handle = range.shared_parent_outside();
        while !parent_handle.is_root() {
            let node = self.state.dom.lookup_node(&parent_handle);
            let container = match node {
                DomNode::Container(container) => container,
                _ => continue,
            };
            if matches!(container.kind(), ContainerNodeKind::Link(_)) {
                return Some(node.handle());
            }
            parent_handle = parent_handle.parent_handle();
        }

        None
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
                    if !during.is_empty() {
                        new_nodes.push(DomNode::new_link(
                            link.clone(),
                            vec![DomNode::new_text(during)],
                        ));
                    }
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

    pub fn remove_links(&mut self) -> ComposerUpdate<S> {
        let mut has_found_link = false;
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let iter = range.locations.into_iter().rev();
        for loc in iter {
            if loc.kind == DomNodeKind::Link {
                if !has_found_link {
                    has_found_link = true;
                    self.push_state_to_history();
                }
                self.state
                    .dom
                    .replace_node_with_its_children(&loc.node_handle);
            }
        }
        if !has_found_link {
            return ComposerUpdate::keep();
        }
        self.create_update_replace_all()
    }
}
