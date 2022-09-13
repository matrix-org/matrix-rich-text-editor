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

use crate::composer_model::base::{slice_from, slice_to};
use crate::dom::nodes::{ContainerNode, DomNode};
use crate::dom::to_raw_text::ToRawText;
use crate::dom::{DomHandle, Range, SameNodeRange};
use crate::{ComposerModel, ComposerUpdate, ListType, Location, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn ordered_list(&mut self) -> ComposerUpdate<S> {
        self.toggle_list(ListType::Ordered)
    }

    pub fn unordered_list(&mut self) -> ComposerUpdate<S> {
        self.toggle_list(ListType::Unordered)
    }

    pub(crate) fn do_backspace_in_list(
        &mut self,
        parent_handle: DomHandle,
        location: usize,
        range: SameNodeRange,
    ) -> ComposerUpdate<S> {
        // do_backspace_in_list should only be called on a single location
        // as selection can be handled in standard do_backspace.
        assert_eq!(range.start_offset, range.end_offset);
        let parent_node = self.state.dom.lookup_node(&parent_handle);
        let list_node_handle = parent_node.handle().parent_handle();
        if let DomNode::Container(parent) = parent_node {
            if parent.is_empty_list_item() {
                // Store current Dom
                self.push_state_to_history();
                self.remove_list_item(
                    list_node_handle,
                    location,
                    parent_handle.index_in_parent(),
                    false,
                );
                self.create_update_replace_all()
            } else {
                self.do_backspace()
            }
        } else {
            panic!("No list item found")
        }
    }

    pub(crate) fn do_enter_in_list(
        &mut self,
        parent_handle: DomHandle,
        location: usize,
        range: SameNodeRange,
    ) -> ComposerUpdate<S> {
        // do_enter_in_list should only be called on a single location
        // as selection can be deleted beforehand.
        assert_eq!(range.start_offset, range.end_offset);
        // Store current Dom
        self.push_state_to_history();
        let parent_node = self.state.dom.lookup_node(&parent_handle);
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

    fn toggle_list(&mut self, list_type: ListType) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);

        match range {
            Range::SameNode(range) => {
                let parent_list_item_handle = self
                    .state
                    .dom
                    .find_parent_list_item(range.node_handle.clone());
                if let Some(list_item_handle) = parent_list_item_handle {
                    let list_node_handle = list_item_handle.parent_handle();
                    let list_node =
                        self.state.dom.lookup_node(&list_node_handle);
                    if let DomNode::Container(list) = list_node {
                        if list.is_list_of_type(list_type.clone()) {
                            self.move_list_item_content_to_list_parent(
                                list_item_handle,
                            )
                        } else {
                            self.update_list_type(list_node_handle, list_type)
                        }
                    } else {
                        panic!("List item is not in a list")
                    }
                } else {
                    self.create_list(list_type)
                }
            }
            Range::NoNode => self.create_list(list_type),
            _ => {
                panic!("Can't toggle list in complex object models yet")
            }
        }
    }

    fn move_list_item_content_to_list_parent(
        &mut self,
        list_item_handle: DomHandle,
    ) -> ComposerUpdate<S> {
        let list_item_node = self.state.dom.lookup_node(&list_item_handle);
        if let DomNode::Container(list_item) = list_item_node {
            let list_item_children = list_item.children().clone();
            let list_handle = list_item_handle.parent_handle();
            let list_index_in_parent = list_handle.index_in_parent();
            let list_parent_handle = list_handle.parent_handle();
            let list_parent_node =
                self.state.dom.lookup_node_mut(&list_parent_handle);
            if let DomNode::Container(list_parent) = list_parent_node {
                for child in list_item_children.iter().rev() {
                    list_parent
                        .insert_child(list_index_in_parent + 1, child.clone());
                }
            } else {
                panic!("List parent node is not a container")
            }

            let list_item_index_in_parent = list_item_handle.index_in_parent();
            let list_node = self.state.dom.lookup_node_mut(&list_handle);
            if let DomNode::Container(list) = list_node {
                list.remove_child(list_item_index_in_parent);
            } else {
                panic!("List node is not a container")
            }
        } else {
            panic!("List item is not a container")
        }

        return self.create_update_replace_all();
    }

    fn update_list_type(
        &mut self,
        list_handle: DomHandle,
        list_type: ListType,
    ) -> ComposerUpdate<S> {
        let list_node = self.state.dom.lookup_node_mut(&list_handle);
        if let DomNode::Container(list) = list_node {
            list.set_list_type(list_type);
        }
        return self.create_update_replace_all();
    }

    fn create_list(&mut self, list_type: ListType) -> ComposerUpdate<S> {
        // Store current Dom
        self.push_state_to_history();
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        match range {
            Range::SameNode(range) => {
                let node = self.state.dom.lookup_node(&range.node_handle);
                if let DomNode::Text(t) = node {
                    let text = t.data();
                    let index_in_parent = range.node_handle.index_in_parent();
                    let list_item =
                        DomNode::Container(ContainerNode::new_list_item(
                            S::from_str("li"),
                            vec![DomNode::new_text(text.clone())],
                        ));
                    if index_in_parent > 0 {
                        let previous_handle = range.node_handle.prev_sibling();
                        let previous_node =
                            self.state.dom.lookup_node_mut(&previous_handle);
                        if let DomNode::Container(previous) = previous_node {
                            if previous.is_list_of_type(list_type.clone()) {
                                previous.append_child(list_item);
                                let parent_node_handle =
                                    range.node_handle.parent_handle();
                                let parent_node = self
                                    .state
                                    .dom
                                    .lookup_node_mut(&parent_node_handle);
                                if let DomNode::Container(parent) = parent_node
                                {
                                    parent.remove_child(index_in_parent);
                                } else {
                                    panic!(
                                        "Unexpected missing parent container"
                                    )
                                }

                                return self.create_update_replace_all();
                            }
                        }
                    }

                    self.replace_node_with_new_list(
                        range.node_handle.clone(),
                        list_type,
                        list_item,
                    );
                    return self.create_update_replace_all();
                } else {
                    panic!("Can't create a list from a non-text node")
                }
            }

            Range::NoNode => {
                self.state.dom.append_child(DomNode::new_list(
                    list_type,
                    vec![DomNode::Container(ContainerNode::new_list_item(
                        S::from_str("li"),
                        vec![DomNode::new_text(S::from_str(""))],
                    ))],
                ));
                return self.create_update_replace_all();
            }

            _ => {
                panic!("Can't create ordered list in complex object models yet")
            }
        }
    }

    fn replace_node_with_new_list(
        &mut self,
        handle: DomHandle,
        list_type: ListType,
        list_item: DomNode<S>,
    ) {
        let list_node = DomNode::new_list(list_type, vec![list_item]);
        self.state.dom.replace(handle, vec![list_node]);
    }

    fn slice_list_item(
        &mut self,
        handle: DomHandle,
        location: usize,
        range: SameNodeRange,
    ) {
        let text_node = self.state.dom.lookup_node_mut(&range.node_handle);
        if let DomNode::Text(ref mut t) = text_node {
            let text = t.data();
            // TODO: should slice container nodes between li and text node as well
            let new_text = slice_to(text, ..range.start_offset);
            let new_li_text = slice_from(text, range.end_offset..);
            t.set_data(new_text);
            let list_node = self.state.dom.lookup_node_mut(&handle);
            if let DomNode::Container(list) = list_node {
                let add_zwsp = new_li_text.len() == 0;
                list.append_child(DomNode::new_list_item(
                    S::from_str("li"),
                    vec![DomNode::new_text(if add_zwsp {
                        S::from_str("\u{200b}")
                    } else {
                        new_li_text
                    })],
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
        let list_node = self.state.dom.lookup_node_mut(&list_handle);
        if let DomNode::Container(list) = list_node {
            let list_len = list.to_raw_text().len();
            let li_len = list.children()[li_index].to_raw_text().len();
            if list.children().len() == 1 {
                let parent_handle = list_handle.parent_handle();
                let parent_node =
                    self.state.dom.lookup_node_mut(&parent_handle);
                if let DomNode::Container(parent) = parent_node {
                    parent.remove_child(list_handle.index_in_parent());
                    if parent.children().len() == 0 {
                        parent.append_child(DomNode::new_text(S::from_str("")));
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
                        self.state.dom.lookup_node_mut(&parent_handle);
                    if let DomNode::Container(parent) = parent_node {
                        // TODO: should probably append a paragraph instead
                        parent.append_child(DomNode::new_text(S::from_str(
                            "\u{200b}",
                        )));
                        let new_location =
                            Location::from(location - li_len + 1);
                        self.state.start = new_location;
                        self.state.end = new_location;
                    } else {
                        panic!("Parent node is not a container")
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
