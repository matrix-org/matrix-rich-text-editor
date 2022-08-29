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

use crate::composer_model::{slice_from, slice_to};
use crate::dom::nodes::{ContainerNode, DomNode, TextNode};
use crate::dom::to_raw_text::ToRawText;
use crate::dom::{DomHandle, Range, SameNodeRange};
use crate::{ComposerModel, ComposerUpdate, Location, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn create_ordered_list(&mut self) -> ComposerUpdate<S> {
        self.create_list(true)
    }

    pub fn create_unordered_list(&mut self) -> ComposerUpdate<S> {
        self.create_list(false)
    }

    pub(crate) fn do_enter_in_list(
        &mut self,
        parent_handle: DomHandle,
        location: usize,
        range: SameNodeRange,
    ) -> ComposerUpdate<S> {
        // Store current Dom
        self.push_state_to_history();
        let parent_node = self.state.dom.lookup_node(parent_handle.clone());
        let list_node_handle = parent_node.handle().parent_handle();
        if let DomNode::Container(parent) = parent_node {
            if parent.is_empty_list_item() {
                self.remove_list_item(
                    list_node_handle,
                    location,
                    parent_handle.index_in_parent(),
                    true,
                );
            } else {
                self.slice_list_item(list_node_handle, location, range);
            }
            self.create_update_replace_all()
        } else {
            panic!("No list item found")
        }
    }

    fn create_list(&mut self, ordered: bool) -> ComposerUpdate<S> {
        // Store current Dom
        self.push_state_to_history();
        let list_tag = if ordered { "ol" } else { "ul" };
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        match range {
            Range::SameNode(range) => {
                let node =
                    self.state.dom.lookup_node(range.node_handle.clone());
                if let DomNode::Text(t) = node {
                    let text = t.data();
                    let list_node = DomNode::new_list(
                        S::from_str(list_tag),
                        vec![DomNode::Container(ContainerNode::new_list_item(
                            S::from_str("li"),
                            vec![DomNode::Text(TextNode::from(text.clone()))],
                        ))],
                    );
                    self.state.dom.replace(range.node_handle, vec![list_node]);
                    return self.create_update_replace_all();
                } else {
                    panic!("Can't create a list from a non-text node")
                }
            }

            Range::NoNode => {
                self.state.dom.append_child(DomNode::new_list(
                    S::from_str(list_tag),
                    vec![DomNode::Container(ContainerNode::new_list_item(
                        S::from_str("li"),
                        vec![DomNode::Text(TextNode::from(S::from_str("")))],
                    ))],
                ));
                return self.create_update_replace_all();
            }

            _ => {
                panic!("Can't create ordered list in complex object models yet")
            }
        }
    }

    fn slice_list_item(
        &mut self,
        handle: DomHandle,
        location: usize,
        range: SameNodeRange,
    ) {
        assert_eq!(range.start_offset, range.end_offset);
        let text_node = self.state.dom.lookup_node_mut(range.node_handle);
        if let DomNode::Text(ref mut t) = text_node {
            let text = t.data();
            // TODO: should slice container nodes between li and text node as well
            let new_text = slice_to(text, ..range.start_offset);
            let new_li_text = slice_from(text, range.end_offset..);
            t.set_data(new_text);
            let list_node = self.state.dom.lookup_node_mut(handle);
            if let DomNode::Container(list) = list_node {
                let add_zwsp = new_li_text.len() == 0;
                list.append_child(DomNode::new_list_item(
                    S::from_str("li"),
                    vec![DomNode::Text(TextNode::from(if add_zwsp {
                        S::from_str("\u{200B}")
                    } else {
                        new_li_text
                    }))],
                ));
                if add_zwsp {
                    self.state.start = Location::from(location + 1);
                    self.state.end = Location::from(location + 1);
                }
            }
        }
    }

    fn remove_list_item(
        &mut self,
        list_handle: DomHandle,
        location: usize,
        li_index: usize,
        insert_trailing_text_node: bool,
    ) {
        let list_node = self.state.dom.lookup_node_mut(list_handle.clone());
        if let DomNode::Container(list) = list_node {
            let list_len = list.to_raw_text().len();
            let li_len = list.children()[li_index].to_raw_text().len();
            if list.children().len() == 1 {
                let parent_handle = list_handle.parent_handle();
                let parent_node = self.state.dom.lookup_node_mut(parent_handle);
                if let DomNode::Container(parent) = parent_node {
                    parent.remove_child(list_handle.index_in_parent());
                    if parent.children().len() == 0 {
                        parent.append_child(DomNode::Text(TextNode::from(
                            S::from_str(""),
                        )));
                    }
                    let new_location = Location::from(location - list_len);
                    self.state.start = new_location;
                    self.state.end = new_location;
                } else {
                    // TODO: handle list items outside of lists
                    panic!("List has no parent container")
                }
            } else {
                list.remove_child(li_index);
                if insert_trailing_text_node {
                    let parent_handle = list_handle.parent_handle();
                    let parent_node =
                        self.state.dom.lookup_node_mut(parent_handle);
                    if let DomNode::Container(parent) = parent_node {
                        // TODO: should probably append a paragraph instead
                        parent.append_child(DomNode::Text(TextNode::from(
                            S::from_str("\u{200B}"),
                        )));
                        let new_location =
                            Location::from(location - li_len + 1);
                        self.state.start = new_location;
                        self.state.end = new_location;
                    } else {
                        panic!("List has no parent container")
                    }
                } else {
                    let new_location = Location::from(location - li_len);
                    self.state.start = new_location;
                    self.state.end = new_location;
                }
            }
        }
    }
}
