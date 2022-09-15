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

use std::collections::HashMap;

use crate::composer_model::base::{slice_from, slice_to};
use crate::dom::nodes::{ContainerNode, ContainerNodeKind, DomNode};
use crate::dom::to_raw_text::ToRawText;
use crate::dom::{DomHandle, DomLocation, Range, SameNodeRange};
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
        parent_handle: &DomHandle,
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
                    &list_node_handle,
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

    /// Insert a newline in some text.
    /// handle is a DomHandle to a text node
    /// offset is the number of code units into the text to insert a newline
    pub(crate) fn do_enter_in_text(
        &mut self,
        handle: &DomHandle,
        offset: usize,
    ) -> ComposerUpdate<S> {
        self.state.dom.insert_into_text(
            handle,
            offset,
            DomNode::new_line_break(),
        );
        self.state.start += 1;
        self.state.end = self.state.start;
        self.create_update_replace_all()
    }

    pub(crate) fn do_enter_in_list(
        &mut self,
        parent_handle: &DomHandle,
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
                    &list_node_handle,
                    location,
                    parent_handle.index_in_parent(),
                    true,
                );
            } else {
                self.slice_list_item(&list_node_handle, location, range);
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
                    .find_parent_list_item(&range.node_handle.clone());
                if let Some(list_item_handle) = parent_list_item_handle {
                    let list_node_handle = list_item_handle.parent_handle();
                    let list_node =
                        self.state.dom.lookup_node(&list_node_handle);
                    if let DomNode::Container(list) = list_node {
                        if list.is_list_of_type(list_type.clone()) {
                            self.move_list_item_content_to_list_parent(
                                &list_item_handle,
                            )
                        } else {
                            self.update_list_type(&list_node_handle, list_type)
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
        list_item_handle: &DomHandle,
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
        list_handle: &DomHandle,
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
                        &range.node_handle,
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
        handle: &DomHandle,
        list_type: ListType,
        list_item: DomNode<S>,
    ) {
        let list_node = DomNode::new_list(list_type, vec![list_item]);
        self.state.dom.replace(&handle, vec![list_node]);
    }

    fn slice_list_item(
        &mut self,
        handle: &DomHandle,
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
        list_handle: &DomHandle,
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

    pub fn can_indent(&self, locations: Vec<DomLocation>) -> bool {
        for loc in locations {
            if loc.is_leaf && !self.can_indent_handle(&loc.node_handle) {
                return false;
            }
        }
        true
    }

    pub(crate) fn can_indent_handle(&self, handle: &DomHandle) -> bool {
        if let DomNode::Container(parent) =
            self.state.dom.lookup_node(&handle.parent_handle())
        {
            let parent = parent;

            // Parent must be a ListItem
            if parent.is_list_item() {
                return handle.parent_handle().index_in_parent() > 0;
            }
        } else {
            panic!("Parent node must be a ContainerNode");
        }
        false
    }

    pub fn can_un_indent(&self, locations: Vec<DomLocation>) -> bool {
        for loc in locations {
            if loc.is_leaf && !self.can_un_indent_handle(&loc.node_handle) {
                return false;
            }
        }
        true
    }

    pub(crate) fn can_un_indent_handle(&self, handle: &DomHandle) -> bool {
        // Check that there are at least 2 ancestor lists
        if let Some(closest_list_handle) =
            self.find_closest_list_ancestor(&handle)
        {
            self.find_closest_list_ancestor(&closest_list_handle)
                .is_some()
        } else {
            false
        }
    }

    fn find_closest_list_ancestor(
        &self,
        handle: &DomHandle,
    ) -> Option<DomHandle> {
        if handle.has_parent() {
            let parent_handle = handle.parent_handle();
            if let DomNode::Container(parent) =
                self.state.dom.lookup_node(&parent_handle)
            {
                if *parent.kind() == ContainerNodeKind::List {
                    return Some(parent_handle.clone());
                } else if parent_handle.has_parent() {
                    return self.find_closest_list_ancestor(&parent_handle);
                }
            }
        }
        None
    }

    pub fn indent(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        match range {
            Range::SameNode(r) => {
                if self.can_indent_handle(&r.node_handle) {
                    self.indent_handles(vec![r.node_handle]);
                    self.create_update_replace_all()
                } else {
                    ComposerUpdate::keep()
                }
            }
            Range::MultipleNodes(r) => {
                if self.can_indent(r.locations.clone()) {
                    self.indent_locations(r.locations);
                    self.create_update_replace_all()
                } else {
                    ComposerUpdate::keep()
                }
            }
            _ => ComposerUpdate::keep(),
        }
    }

    pub fn un_indent(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        match range {
            Range::SameNode(r) => {
                if self.can_un_indent_handle(&r.node_handle) {
                    self.un_indent_handles(vec![r.node_handle]);
                    self.create_update_replace_all()
                } else {
                    ComposerUpdate::keep()
                }
            }
            Range::MultipleNodes(r) => {
                if self.can_un_indent(r.locations.clone()) {
                    self.un_indent_locations(r.locations);
                    self.create_update_replace_all()
                } else {
                    ComposerUpdate::keep()
                }
            }
            _ => ComposerUpdate::keep(),
        }
    }

    fn indent_locations(&mut self, locations: Vec<DomLocation>) {
        self.indent_handles(Self::leaf_handles_from_locations(locations));
    }

    fn indent_handles(&mut self, handles: Vec<DomHandle>) {
        let by_list_sorted = Self::group_sorted_handles_by_list_parent(handles);

        for (parent_handle, handles) in by_list_sorted.iter().rev() {
            let mut sorted_handles = handles.clone();
            sorted_handles.sort();

            let parent_list_type = if let DomNode::Container(list) =
                self.state.dom.lookup_node(&parent_handle)
            {
                if list.is_list_of_type(ListType::Ordered) {
                    ListType::Ordered
                } else {
                    ListType::Unordered
                }
            } else {
                panic!("ListItem parent must be a ContainerNode");
            };

            let first_handle = sorted_handles.first().unwrap();
            let into_handle = if let Some(h) = self
                .get_last_list_node_in_list_item(
                    &first_handle.parent_handle().prev_sibling(),
                    &parent_list_type,
                ) {
                h.handle()
            } else {
                first_handle.parent_handle().prev_sibling()
            };

            let at_index = if let DomNode::Container(into_node) =
                self.state.dom.lookup_node(&into_handle)
            {
                into_node.children().len()
            } else {
                0
            };

            for handle in handles.iter().rev() {
                self.indent_single_handle(
                    &handle,
                    &into_handle,
                    at_index,
                    &parent_list_type,
                );
            }
        }
    }

    fn indent_single_handle(
        &mut self,
        handle: &DomHandle,
        into_handle: &DomHandle,
        at_index: usize,
        parent_list_type: &ListType,
    ) {
        let list_item_handle = handle.parent_handle();
        if list_item_handle.index_in_parent() == 0 {
            panic!("Can't indent first list item node");
        }
        let removed_list_item = if let DomNode::Container(list_item) =
            self.state.dom.lookup_node(&list_item_handle)
        {
            if !list_item.is_list_item() {
                panic!("Parent node must be a list item");
            }
            if let DomNode::Container(list) = self
                .state
                .dom
                .lookup_node_mut(&list_item_handle.parent_handle())
            {
                list.remove_child(list_item_handle.index_in_parent())
            } else {
                panic!("ListItem mus have a parent");
            }
        } else {
            panic!("Parent must be a ContainerNode");
        };

        if let DomNode::Container(into_node) =
            self.state.dom.lookup_node_mut(&into_handle)
        {
            // New list node added here, insert it into that container at index 0
            if at_index < into_node.children().len() {
                if let Some(DomNode::Container(sub_node)) =
                    into_node.get_child_mut(at_index)
                {
                    sub_node.insert_child(0, removed_list_item);
                }
            } else if into_node.is_list_of_type(parent_list_type.clone()) {
                into_node.insert_child(at_index, removed_list_item);
            } else {
                let new_list = DomNode::new_list(
                    parent_list_type.clone(),
                    vec![removed_list_item],
                );
                into_node.insert_child(at_index, new_list);
            }
        }
    }

    fn get_last_list_node_in_list_item(
        &mut self,
        handle: &DomHandle,
        list_type: &ListType,
    ) -> Option<&mut ContainerNode<S>> {
        if let DomNode::Container(prev_sibling) =
            self.state.dom.lookup_node_mut(&handle)
        {
            if !prev_sibling.children().is_empty() {
                if let DomNode::Container(prev_sibling_last_item) = prev_sibling
                    .get_child_mut(prev_sibling.children().len() - 1)
                    .unwrap()
                {
                    if prev_sibling_last_item.is_list_of_type(list_type.clone())
                    {
                        return Some(prev_sibling_last_item);
                    }
                }
            }
        }
        None
    }

    pub fn un_indent_locations(&mut self, locations: Vec<DomLocation>) {
        self.un_indent_handles(Self::leaf_handles_from_locations(locations));
    }

    fn un_indent_handles(&mut self, handles: Vec<DomHandle>) {
        let by_list_sorted = Self::group_sorted_handles_by_list_parent(handles);

        for (list_handle, handles) in by_list_sorted.iter().rev() {
            let mut sorted_handles = handles.clone();
            sorted_handles.sort();

            let at_index = list_handle.parent_handle().index_in_parent() + 1;
            if let Some(into_handle) =
                self.find_closest_list_ancestor(list_handle)
            {
                for handle in handles.iter().rev() {
                    self.un_indent_single_handle(
                        handle,
                        &into_handle,
                        at_index,
                    );
                }
            } else {
                panic!("Current list should have another list ancestor");
            }
        }
    }

    fn un_indent_single_handle(
        &mut self,
        handle: &DomHandle,
        into_handle: &DomHandle,
        at_index: usize,
    ) {
        let list_item_handle = handle.parent_handle();
        let mut list_node_to_insert = None;
        let removed_list_item = if let DomNode::Container(current_parent) = self
            .state
            .dom
            .lookup_node_mut(&list_item_handle.parent_handle())
        {
            if current_parent.children().len() > 1 {
                let mut to_add = Vec::new();
                let from = list_item_handle.index_in_parent() + 1;
                for i in (from..current_parent.children().len()).rev() {
                    to_add.insert(0, current_parent.remove_child(i));
                }
                if !to_add.is_empty() {
                    let list_type =
                        ListType::from(current_parent.name().clone());
                    list_node_to_insert =
                        Some(DomNode::new_list(list_type, to_add));
                }
            }
            let ret =
                current_parent.remove_child(list_item_handle.index_in_parent());
            if current_parent.children().is_empty() {
                // List is empty, remove list node
                self.state
                    .dom
                    .replace(list_item_handle.parent_handle(), Vec::new());
            }
            ret
        } else {
            panic!("Handle {:?} has no parent", handle);
        };

        if let DomNode::Container(new_list_parent) =
            self.state.dom.lookup_node_mut(&into_handle)
        {
            new_list_parent.insert_child(at_index, removed_list_item);

            if let Some(list_with_remnants) = list_node_to_insert {
                if let Some(DomNode::Container(inserted_list_item)) =
                    new_list_parent.get_child_mut(at_index)
                {
                    inserted_list_item.append_child(list_with_remnants);
                }
            }
        } else {
            panic!("New list parent must be a ContainerNode");
        }
    }

    fn leaf_handles_from_locations(
        locations: Vec<DomLocation>,
    ) -> Vec<DomHandle> {
        locations
            .iter()
            .filter_map(|l| {
                if l.is_leaf {
                    Some(l.node_handle.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn group_sorted_handles_by_list_parent(
        handles: Vec<DomHandle>,
    ) -> Vec<(DomHandle, Vec<DomHandle>)> {
        let mut by_list: HashMap<DomHandle, Vec<DomHandle>> = HashMap::new();
        for handle in handles.iter() {
            // If it's a leaf node, it should be Text > ListItem > List
            let list_handle = handle.parent_handle().parent_handle();
            if let Some(list) = by_list.get_mut(&list_handle) {
                list.push(handle.clone());
            } else {
                by_list.insert(list_handle, vec![handle.clone()]);
            }
        }

        let mut by_list_sorted: Vec<(DomHandle, Vec<DomHandle>)> = by_list
            .iter()
            .map(|(h, l)| (h.clone(), l.clone()))
            .collect();
        by_list_sorted.sort();
        by_list_sorted
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::testutils_composer_model::{cm, tx};

    use super::*;

    #[test]
    fn cannot_indent_first_item() {
        let model = cm("<ul><li>{Test}|</li></ul>");
        assert_eq!(
            false,
            model.can_indent_handle(&DomHandle::from_raw(vec![0, 0, 0]))
        );
    }

    #[test]
    fn can_indent_second_item() {
        let model = cm("<ul><li>First item</li><li>{Second item}|</li></ul>");
        assert_eq!(
            true,
            model.can_indent_handle(&DomHandle::from_raw(vec![0, 1, 0]))
        );
    }

    #[test]
    fn can_indent_several_items_if_first_is_not_included() {
        let model = cm("<ul><li>First item</li><li>{Second item</li><li>Third item}|</li></ul>");
        let locations = get_range_locations(&model);
        assert_eq!(true, model.can_indent(locations));
    }

    #[test]
    fn cannot_indent_several_items_if_first_is_included() {
        let model = cm("<ul><li>{First item</li><li>Second item</li><li>Third item}|</li></ul>");
        let locations = get_range_locations(&model);
        assert_eq!(false, model.can_indent(locations));
    }

    #[test]
    fn indent_list_item_works() {
        let mut model = cm("<ul><li>First item</li><li>Second item</li><li>Third item|</li></ul>");
        model.indent_handles(vec![DomHandle::from_raw(vec![0, 1, 0])]);
        assert_eq!(tx(&model), "<ul><li>First item<ul><li>Second item</li></ul></li><li>Third item|</li></ul>");
    }

    #[test]
    fn indent_list_item_to_previous_works() {
        let mut model = cm("<ul><li>First item<ul><li>Second item</li></ul></li><li>Third item|</li></ul>");
        model.indent_handles(vec![DomHandle::from_raw(vec![0, 1, 0])]);
        assert_eq!(tx(&model), "<ul><li>First item<ul><li>Second item</li><li>Third item|</li></ul></li></ul>");
    }

    #[test]
    fn can_un_indent_handle_simple_case_works() {
        let model = cm("<ul><li>First item<ul><li>{Second item</li><li>Third item}|</li></ul></li></ul>");
        let handle = DomHandle::from_raw(vec![0, 0, 1, 0, 0]);
        assert_eq!(true, model.can_un_indent_handle(&handle));
    }

    #[test]
    fn can_un_indent_simple_case_works() {
        let model = cm("<ul><li>First item<ul><li>{Second item</li><li>Third item}|</li></ul></li></ul>");
        let locations = get_range_locations(&model);
        assert_eq!(true, model.can_un_indent(locations));
    }

    #[test]
    fn can_un_indent_with_only_one_list_level_fails() {
        let model = cm("<ul><li>First item</li><li>{Second item</li><li>Third item}|</li></ul>");
        let locations = get_range_locations(&model);
        assert_eq!(false, model.can_un_indent(locations));
    }

    #[test]
    fn un_indent_handles_simple_case_works() {
        let mut model =
            cm("<ul><li>First item<ul><li>{Second item}|</li></ul></li></ul>");
        let handles = vec![DomHandle::from_raw(vec![0, 0, 1, 0, 0])];
        model.un_indent_handles(handles);
        assert_eq!(
            tx(&model),
            "<ul><li>First item</li><li>{Second item}|</li></ul>"
        )
    }

    fn get_range_locations<S: UnicodeString>(
        model: &ComposerModel<S>,
    ) -> Vec<DomLocation> {
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);
        if let Range::MultipleNodes(r) = range {
            r.locations
        } else {
            panic!("No multiple node range found");
        }
    }
}
