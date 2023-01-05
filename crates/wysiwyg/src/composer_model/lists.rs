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

use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::{ContainerNode, DomNode};
use crate::dom::range::DomLocationPosition;
use crate::dom::{DomHandle, DomLocation, Range};
use crate::{ComposerModel, ComposerUpdate, ListType, Location, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn ordered_list(&mut self) -> ComposerUpdate<S> {
        self.push_state_to_history();
        self.toggle_list(ListType::Ordered)
    }

    pub fn unordered_list(&mut self) -> ComposerUpdate<S> {
        self.push_state_to_history();
        self.toggle_list(ListType::Unordered)
    }

    pub fn indent(&mut self) -> ComposerUpdate<S> {
        // push_state_to_history is called if we can indent
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        if !range.locations.is_empty() && self.can_indent(&range.locations) {
            self.push_state_to_history();
            self.indent_locations(&range.locations);
            self.create_update_replace_all()
        } else {
            ComposerUpdate::keep()
        }
    }

    pub fn unindent(&mut self) -> ComposerUpdate<S> {
        // push_state_to_history is called if we can unindent
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        if self.can_unindent(&range.locations) {
            self.push_state_to_history();
            self.unindent_locations(&range.locations);
            self.create_update_replace_all()
        } else {
            ComposerUpdate::keep()
        }
    }

    pub fn can_indent(&self, locations: &Vec<DomLocation>) -> bool {
        if locations.is_empty() {
            return false;
        }
        for loc in locations {
            if loc.is_leaf() && !self.can_indent_handle(&loc.node_handle) {
                return false;
            }
        }
        true
    }

    pub fn can_unindent(&self, locations: &Vec<DomLocation>) -> bool {
        if locations.is_empty() {
            return false;
        }
        for loc in locations {
            if loc.is_leaf() && !self.can_unindent_handle(&loc.node_handle) {
                return false;
            }
        }
        true
    }

    pub(crate) fn do_backspace_in_list(
        &mut self,
        parent_handle: &DomHandle,
    ) -> ComposerUpdate<S> {
        let parent_node = self.state.dom.lookup_node(parent_handle);
        let list_node_handle = parent_node.handle().parent_handle();
        if let DomNode::Container(parent) = parent_node {
            if parent.is_empty_list_item() {
                self.state.dom.extract_list_items(
                    &list_node_handle,
                    parent_handle.index_in_parent(),
                    1,
                );
            }
            self.do_backspace()
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
        list_item_handle: &DomHandle,
        list_item_end_offset: usize,
    ) -> ComposerUpdate<S> {
        // Store current Dom
        self.push_state_to_history();
        let list_item_node = self.state.dom.lookup_node(list_item_handle);
        let list_handle = list_item_node.handle().parent_handle();
        if let DomNode::Container(list_item_node) = list_item_node {
            if list_item_node.is_empty_list_item() {
                // Pressing enter in an empty list item means you want to
                // end the list.
                self.state.dom.extract_list_items(
                    &list_handle,
                    list_item_handle.index_in_parent(),
                    1,
                );
            } else {
                // Pressing enter in a non-empty list item splits this item
                // into two.
                self.state
                    .dom
                    .slice_list_item(list_item_handle, list_item_end_offset);
                // Slicing always adds a single ZWSP.
                self.increment_selection(1);
            }
            self.create_update_replace_all()
        } else {
            panic!("No list item found")
        }
    }

    fn toggle_list(&mut self, list_type: ListType) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        // If the DOM is empty, create an initial text node that will be used to create the list afterwards.
        if self.state.dom.children().is_empty() {
            self.state
                .dom
                .append_at_end_of_document(DomNode::new_empty_text());
        }
        let range = self.state.dom.find_extended_range(s, e);
        self.toggle_list_range(list_type, range)
    }

    fn toggle_list_range(
        &mut self,
        list_type: ListType,
        range: Range,
    ) -> ComposerUpdate<S> {
        if range
            .locations
            .iter()
            // FIXME: filtering positions that are before start, these shouldn't be returned from a > 0 range
            .filter(|l| !(l.relative_position() == DomLocationPosition::Before))
            .any(|l| l.kind == DomNodeKind::List)
        {
            let leaves: Vec<&DomLocation> = range.leaves().collect();
            let leaves_without_zwsp: Vec<_> = leaves
                .iter()
                .filter(|l| l.kind != DomNodeKind::Zwsp)
                .collect();
            // FIXME: Workaround for toggling list when only ZWSP is selected
            if leaves_without_zwsp.len() == 1 {
                let handle = &leaves_without_zwsp[0].node_handle;
                self.single_leave_list_toggle(list_type, handle)
            } else if leaves.len() == 1 {
                let handle = &leaves[0].node_handle;
                self.single_leave_list_toggle(list_type, handle)
            } else {
                // TODO: handle cases where a list is already present in the extended selection.
                panic!("Partially creating/removing list is not handled yet")
            }
        } else {
            self.create_list_from_range(list_type, range)
        }
    }

    // FIXME: remove this function when toggle_list_range handles updating/removing
    fn single_leave_list_toggle(
        &mut self,
        list_type: ListType,
        handle: &DomHandle,
    ) -> ComposerUpdate<S> {
        let parent_list_item_handle =
            self.state.dom.find_parent_list_item_or_self(handle);
        if let Some(list_item_handle) = parent_list_item_handle {
            let list = self.state.dom.parent(&list_item_handle);
            if list.is_list_of_type(&list_type) {
                self.state.dom.extract_list_items(
                    &list_item_handle.parent_handle(),
                    list_item_handle.index_in_parent(),
                    1,
                );
                self.create_update_replace_all()
            } else {
                let list_node_handle = list.handle();
                self.update_list_type(&list_node_handle, list_type)
            }
        } else {
            unreachable!("No list in range. Should have been catched by toggle_list_range")
        }
    }

    fn create_list_from_range(
        &mut self,
        list_type: ListType,
        range: Range,
    ) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let start_correction;
        let end_correction;
        let handles: Vec<&DomHandle> = range
            .top_level_locations()
            // FIXME: filtering positions that are before start, these shouldn't be returned from a > 0 range
            .filter(|l| !(l.relative_position() == DomLocationPosition::Before))
            .map(|l| &l.node_handle)
            .collect();

        let first_handle = handles[0];
        let first_node = self.state.dom.lookup_node(first_handle);
        // It's expected to add a ZWSP for each line break + an additional leading ZWSP.
        // end_correction is always 1, start_correction is 1 only if the start is located
        // strictly after the first char of the range (or if we are creating a list from an
        // empty text node, in that case the first node text_len is 0). If a leading ZWSP
        // is already present, e.g. the node following another list both corrections are 0.
        // If a leading line break is present it's gonna be replaced by the first ZWSP, so the
        // correction should be 0 too.
        if first_node.has_leading_zwsp() || first_node.has_leading_line_break()
        {
            start_correction = 0;
            end_correction = 0;
        } else {
            start_correction = usize::from(
                s - range.start() > 0 || first_node.text_len() == 0,
            );
            end_correction = 1;
        }

        self.state.dom.wrap_nodes_in_list(list_type, handles);
        self.select(
            Location::from(s + start_correction),
            Location::from(e + end_correction),
        );
        self.create_update_replace_all()
    }

    fn update_list_type(
        &mut self,
        list_handle: &DomHandle,
        list_type: ListType,
    ) -> ComposerUpdate<S> {
        let list_node = self.state.dom.lookup_node_mut(list_handle);
        if let DomNode::Container(list) = list_node {
            list.set_list_type(list_type);
        }
        self.create_update_replace_all()
    }

    pub(crate) fn can_indent_handle(&self, handle: &DomHandle) -> bool {
        let parent = self.state.dom.parent(handle);
        if parent.is_list_item() {
            handle.parent_handle().index_in_parent() > 0
        } else {
            false
        }
    }

    pub(crate) fn can_unindent_handle(&self, handle: &DomHandle) -> bool {
        // Check that there are at least 2 ancestor lists
        if let Some(closest_list_handle) =
            self.state.dom.find_closest_list_ancestor(handle)
        {
            self.state
                .dom
                .find_closest_list_ancestor(&closest_list_handle)
                .is_some()
        } else {
            false
        }
    }

    fn indent_locations(&mut self, locations: &[DomLocation]) {
        self.indent_handles(&Self::leaf_handles_from_locations(locations));
    }

    fn indent_handles(&mut self, handles: &[DomHandle]) {
        let by_list_sorted = Self::group_sorted_handles_by_list_parent(handles);

        for (parent_handle, handles) in by_list_sorted.iter().rev() {
            let mut sorted_handles = handles.clone();
            sorted_handles.sort();

            let parent_list_type = if let DomNode::Container(list) =
                self.state.dom.lookup_node(parent_handle)
            {
                if list.is_list_of_type(&ListType::Ordered) {
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
                    handle,
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
        let list_item = self.state.dom.parent(handle);
        let list_item_handle = list_item.handle();
        if list_item_handle.index_in_parent() == 0 {
            panic!("Can't indent first list item node");
        }
        if !list_item.is_list_item() {
            panic!("Parent node must be a list item");
        }
        let list = self.state.dom.parent_mut(&list_item_handle);
        let removed_list_item =
            list.remove_child(list_item_handle.index_in_parent());

        if let DomNode::Container(into_node) =
            self.state.dom.lookup_node_mut(into_handle)
        {
            // New list node added here, insert it into that container at index 0
            if at_index < into_node.children().len() {
                if let Some(DomNode::Container(sub_node)) =
                    into_node.get_child_mut(at_index)
                {
                    sub_node.insert_child(0, removed_list_item);
                }
            } else if into_node.is_list_of_type(parent_list_type) {
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
            self.state.dom.lookup_node_mut(handle)
        {
            if !prev_sibling.children().is_empty() {
                if let DomNode::Container(prev_sibling_last_item) = prev_sibling
                    .get_child_mut(prev_sibling.children().len() - 1)
                    .unwrap()
                {
                    if prev_sibling_last_item.is_list_of_type(list_type) {
                        return Some(prev_sibling_last_item);
                    }
                }
            }
        }
        None
    }

    fn unindent_locations(&mut self, locations: &[DomLocation]) {
        self.unindent_handles(&Self::leaf_handles_from_locations(locations));
    }

    fn unindent_handles(&mut self, handles: &[DomHandle]) {
        let by_list_sorted = Self::group_sorted_handles_by_list_parent(handles);

        for (list_handle, handles) in by_list_sorted.iter().rev() {
            let mut sorted_handles = handles.clone();
            sorted_handles.sort();

            let at_index = list_handle.parent_handle().index_in_parent() + 1;
            if let Some(into_handle) =
                self.state.dom.find_closest_list_ancestor(list_handle)
            {
                for handle in handles.iter().rev() {
                    self.unindent_single_handle(handle, &into_handle, at_index);
                }
            } else {
                panic!("Current list should have another list ancestor");
            }
        }
    }

    fn unindent_single_handle(
        &mut self,
        handle: &DomHandle,
        into_handle: &DomHandle,
        at_index: usize,
    ) {
        let list_item_handle = handle.parent_handle();
        let mut list_node_to_insert = None;
        let current_parent = self.state.dom.parent_mut(&list_item_handle);
        if current_parent.children().len() > 1 {
            let mut to_add = Vec::new();
            let from = list_item_handle.index_in_parent() + 1;
            for i in (from..current_parent.children().len()).rev() {
                to_add.insert(0, current_parent.remove_child(i));
            }
            if !to_add.is_empty() {
                let list_type =
                    ListType::from(current_parent.name().to_owned());
                list_node_to_insert =
                    Some(DomNode::new_list(list_type, to_add));
            }
        }
        let removed_list_item =
            current_parent.remove_child(list_item_handle.index_in_parent());
        if current_parent.children().is_empty() {
            // List is empty, remove list node
            self.state
                .dom
                .replace(&list_item_handle.parent_handle(), Vec::new());
        }

        if let DomNode::Container(new_list_parent) =
            self.state.dom.lookup_node_mut(into_handle)
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
        locations: &[DomLocation],
    ) -> Vec<DomHandle> {
        locations
            .iter()
            .filter_map(|l| {
                if l.is_leaf() && l.kind != DomNodeKind::Zwsp {
                    Some(l.node_handle.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    fn group_sorted_handles_by_list_parent(
        handles: &[DomHandle],
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
        assert!(!model.can_indent_handle(&DomHandle::from_raw(vec![0, 0, 0])));
    }

    #[test]
    fn can_indent_second_item() {
        let model = cm("<ul><li>First item</li><li>{Second item}|</li></ul>");
        assert!(model.can_indent_handle(&DomHandle::from_raw(vec![0, 1, 0])));
    }

    #[test]
    fn can_indent_several_items_if_first_is_not_included() {
        let model = cm("<ul><li>First item</li><li>{Second item</li><li>Third item}|</li></ul>");
        let locations = get_range_locations(&model);
        assert!(model.can_indent(&locations));
    }

    #[test]
    fn cannot_indent_several_items_if_first_is_included() {
        let model = cm("<ul><li>{First item</li><li>Second item</li><li>Third item}|</li></ul>");
        let locations = get_range_locations(&model);
        assert!(!model.can_indent(&locations));
    }

    #[test]
    fn indent_list_item_works() {
        let mut model = cm("<ul><li>First item</li><li>Second item</li><li>Third item|</li></ul>");
        model.indent_handles(&[DomHandle::from_raw(vec![0, 1, 0])]);
        assert_eq!(tx(&model), "<ul><li>First item<ul><li>Second item</li></ul></li><li>Third item|</li></ul>");
    }

    #[test]
    fn indent_list_item_to_previous_works() {
        let mut model = cm("<ul><li>First item<ul><li>Second item</li></ul></li><li>Third item|</li></ul>");
        model.indent_handles(&[DomHandle::from_raw(vec![0, 1, 0])]);
        assert_eq!(tx(&model), "<ul><li>First item<ul><li>Second item</li><li>Third item|</li></ul></li></ul>");
    }

    #[test]
    fn can_unindent_handle_simple_case_works() {
        let model = cm("<ul><li>First item<ul><li>{Second item</li><li>Third item}|</li></ul></li></ul>");
        let handle = DomHandle::from_raw(vec![0, 0, 1, 0, 0]);
        assert!(model.can_unindent_handle(&handle));
    }

    #[test]
    fn can_unindent_simple_case_works() {
        let model = cm("<ul><li>First item<ul><li>{Second item</li><li>Third item}|</li></ul></li></ul>");
        let locations = get_range_locations(&model);
        assert!(model.can_unindent(&locations));
    }

    #[test]
    fn can_unindent_with_only_one_list_level_fails() {
        let model = cm("<ul><li>First item</li><li>{Second item</li><li>Third item}|</li></ul>");
        let locations = get_range_locations(&model);
        assert!(!model.can_unindent(&locations));
    }

    #[test]
    fn unindent_handles_simple_case_works() {
        let mut model =
            cm("<ul><li>First item<ul><li>{Second item}|</li></ul></li></ul>");
        let handles = vec![DomHandle::from_raw(vec![0, 0, 1, 0, 0])];
        model.unindent_handles(&handles);
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
        range.locations
    }
}
