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
use crate::dom::{DomLocation, Range, ToRawText};
use crate::{
    ComposerModel, ComposerUpdate, LinkAction, Location, UnicodeString,
};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    // Get the link action
    pub fn get_link_action(&self) -> LinkAction<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let locations = range.locations;

        // If the selection is within a single link then we can edit it
        let link_locations: Vec<DomLocation> = locations
            .into_iter()
            .filter(|loc| loc.kind == DomNodeKind::Link)
            .collect();
        if link_locations.len() == 1 {
            let link = &link_locations[0];
            // TODO: extract this logic to a function
            if s >= link.index_in_dom()
                && e <= link.index_in_dom() + link.length
            {
                let node = self.state.dom.lookup_node(&link.node_handle);
                let text = node.to_raw_text();
                let link = node.as_container().unwrap().get_link().unwrap();
                return LinkAction::Edit { link, text };
            }
        }

        if s == e {
            LinkAction::Insert
        } else {
            LinkAction::Create
        }
    }
    // Inserts some text with a link
    //   If there is a selection, the selected text is replaced
    //   If there is no selection, the new linked text is inserted at the cursor position
    pub fn insert_link(&mut self, link: S, text: S) -> ComposerUpdate<S> {
        match self.get_link_action() {
            LinkAction::Edit { link: _, text: _ } => {
                panic!("Cannot insert a link inside another link");
            }
            _ => {}
        }
        let (s, _) = self.safe_selection();
        self.push_state_to_history();
        self.do_replace_text(text.clone());
        let e = s + text.len();
        let range = self.state.dom.find_range(s, e);
        self.set_link_range(range, link)
    }

    // Edit the link that is selected or at the cursor position
    pub fn edit_link(&mut self, link: S, text: S) -> ComposerUpdate<S> {
        let (mut s, mut e) = self.safe_selection();
        self.push_state_to_history();
        let range = self.state.dom.find_range(s, e);
        let locations = range.locations;
        for loc in locations {
            if loc.kind == DomNodeKind::Link {
                s = loc.position;
                e = s + loc.length;
                break;
            }
        }

        if s == e {
            // No link was found
            return ComposerUpdate::keep();
        }

        self.select(Location::from(s), Location::from(e));
        self.do_replace_text(text.clone());
        e = s + text.len();
        let range = self.state.dom.find_range(s, e);
        self.set_link_range(range, link)
    }

    // Create a link on the selected text
    pub fn create_link(&mut self, link: S) -> ComposerUpdate<S> {
        match self.get_link_action() {
            LinkAction::Edit { link: _, text: _ } => {
                panic!("Cannot create a link inside another link");
            }
            _ => {}
        }
        self.push_state_to_history();
        let (s, e) = self.safe_selection();

        if s == e {
            panic!("A link can only be created when text is selected");
        }

        self.remove_links();

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

impl<S> ComposerModel<S> where S: UnicodeString {}
