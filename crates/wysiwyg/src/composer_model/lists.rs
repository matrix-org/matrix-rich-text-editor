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
use crate::dom::range::DomLocationPosition::Before;
use crate::dom::{DomHandle, DomLocation, Range};
use crate::{ComposerModel, ComposerUpdate, ListType, UnicodeString};

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
        let locations: Vec<DomLocation> = range
            .locations
            .into_iter()
            .filter(|l| l.relative_position() != Before)
            .collect();
        if !locations.is_empty() && self.can_indent(&locations) {
            self.push_state_to_history();
            self.indent_locations(&locations);
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
            if loc.relative_position() == Before {
                continue;
            }
            if loc.kind == DomNodeKind::ListItem
                && !self.can_indent_list_item_handle(&loc.node_handle)
            {
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
                self.offset_selection(1);
            }
            self.create_update_replace_all()
        } else {
            panic!("No list item found")
        }
    }

    fn toggle_list(&mut self, list_type: ListType) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_extended_range(s, e);
        self.toggle_list_range(list_type, range)
    }

    fn toggle_list_range(
        &mut self,
        list_type: ListType,
        range: Range,
    ) -> ComposerUpdate<S> {
        let list_loc_in_range =
            range.locations.iter().find(|l| l.kind == DomNodeKind::List);
        let list_is_before_selection = list_loc_in_range.map_or(false, |l| {
            l.relative_position() == DomLocationPosition::Before
        });
        let list_is_last_node_in_selection =
            if let Some(list_loc) = list_loc_in_range {
                !range.contains(&list_loc.node_handle.next_sibling())
            } else {
                false
            };
        if list_loc_in_range.is_some()
            && (!list_is_before_selection || list_is_last_node_in_selection)
        {
            let leaves: Vec<&DomLocation> = range.leaves().collect();
            // FIXME: Workaround for toggling list when only ZWSP is selected
            if leaves.len() == 1 {
                let handle = &leaves[0].node_handle;
                self.single_leaf_list_toggle(list_type, handle)
            } else if let Some(block_location) = range.deepest_block_node(None)
            {
                if block_location.length == 0 {
                    self.single_leaf_list_toggle(
                        list_type,
                        &block_location.node_handle,
                    )
                } else {
                    panic!(
                        "This block location can't be turned into a list item"
                    )
                }
            } else {
                // TODO: handle cases where a list is already present in the extended selection.
                panic!("Partially creating/removing list is not handled yet")
            }
        } else {
            self.create_list_from_range(list_type, range)
        }
    }

    // FIXME: remove this function when toggle_list_range handles updating/removing
    fn single_leaf_list_toggle(
        &mut self,
        list_type: ListType,
        handle: &DomHandle,
    ) -> ComposerUpdate<S> {
        let parent_list_item_handle =
            self.state.dom.find_parent_list_item_or_self(handle);
        if let Some(list_item_handle) = parent_list_item_handle {
            let list = self.state.dom.parent(&list_item_handle);
            if list.is_list_of_type(&list_type) {
                if list.children().len() == 1 {
                    // ZWSP is removed, selection should be decremented
                    // In single leaf case, selection is always inside so
                    // both start and end should be decremented.
                    self.offset_selection(-1);
                }

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
        let handles: Vec<&DomHandle> = range
            .top_level_locations()
            // FIXME: filtering positions that are before start, these shouldn't be returned from a > 0 range
            .filter(|l| !(l.relative_position() == DomLocationPosition::Before))
            .map(|l| &l.node_handle)
            .collect();
        self.state.dom.wrap_nodes_in_list(list_type, handles);
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

    pub(crate) fn can_indent_list_item_handle(
        &self,
        handle: &DomHandle,
    ) -> bool {
        let node = self.state.dom.lookup_node(&handle);
        if node.is_list_item() {
            handle.index_in_parent() > 0
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
        let list_item_locations: Vec<&DomLocation> = locations
            .into_iter()
            .filter(|l| l.kind == DomNodeKind::ListItem)
            .collect();
        let top_most_level = list_item_locations
            .iter()
            .map(|l| l.node_handle.depth())
            .min()
            .expect("Couldn't find the depth of any list item");
        let top_most_list_items: Vec<DomHandle> = list_item_locations
            .into_iter()
            .filter(|l| l.node_handle.depth() == top_most_level)
            .map(|l| l.node_handle.clone())
            .collect();

        self.indent_list_item_handles(&top_most_list_items);
    }

    fn indent_list_item_handles(&mut self, handles: &Vec<DomHandle>) {
        if handles.is_empty() {
            return;
        }
        let can_indent =
            handles.iter().all(|h| self.can_indent_list_item_handle(&h));
        if !can_indent {
            return;
        }

        let parent_handle = handles[0].parent_handle();
        let all_have_same_parent =
            handles.iter().all(|h| h.parent_handle() == parent_handle);
        if !all_have_same_parent {
            return;
        }

        let mut sorted_handles = handles.clone();
        sorted_handles.sort();

        let first_handle = sorted_handles.get(0).unwrap();
        let insert_into_handle = first_handle.prev_sibling();

        let parent_list_type = self
            .state
            .dom
            .parent(&first_handle)
            .get_list_type()
            .unwrap()
            .clone();

        let mut removed_list_items = Vec::new();
        for handle in sorted_handles.iter().rev() {
            removed_list_items.insert(0, self.state.dom.remove(&handle));
        }

        if let DomNode::Container(dest_list_item) =
            &mut self.state.dom.lookup_node_mut(&insert_into_handle)
        {
            if dest_list_item.children().len() == 1
                && !dest_list_item.get_child(0).unwrap().is_block_node()
            {
                let children = dest_list_item.remove_children();
                let paragraph = DomNode::new_paragraph(children);
                dest_list_item.append_child(paragraph);
            }

            dest_list_item.append_child(DomNode::new_list(
                parent_list_type,
                removed_list_items,
            ));
        } else {
            panic!("Destination list item must be a container");
        }
        self.state.dom.join_nodes_in_container(&insert_into_handle);
    }

    fn indent_single_handle(
        &mut self,
        // TODO: this handle should be the list_item_handle
        list_item_handle: &DomHandle,
        parent_list_type: &ListType,
    ) {
        // let list_item = self.state.dom.parent(list_item_handle);
        // if list_item_handle.index_in_parent() == 0 {
        //     panic!("Can't indent first list item node");
        // }
        // if !list_item.is_list_item() {
        //     panic!("Parent node must be a list item");
        // }
        // let removed_list_item = self.state.dom.remove(&list_item_handle);
        //
        // if let DomNode::Container(into_node) =
        //     self.state.dom.lookup_node_mut(into_handle)
        // {
        //     if into_node.is_list() {
        //         // Move the list item to an already indented list
        //         into_node.insert_child(at_index, removed_list_item);
        //     } else {
        //         // Is list item:
        //         // 1. Move the existing children into a paragraph if needed.
        //         // 2. Add a new list at the end of this node, containing the removed list item.
        //         let previous_children = into_node.remove_children();
        //         let children =
        //             if previous_children.iter().all(|n| n.is_block_node()) {
        //                 previous_children
        //             } else {
        //                 vec![DomNode::new_paragraph(previous_children)]
        //             };
        //         into_node.append_children(children);
        //         into_node.append_child(DomNode::new_list(
        //             parent_list_type.clone(),
        //             vec![removed_list_item],
        //         ));
        //     }
        // } else {
        //     panic!("into_node must exist and be a container node");
        // }
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
                self.state.advance_selection();
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
        assert!(!model
            .can_indent_list_item_handle(&DomHandle::from_raw(vec![0, 0])));
    }

    #[test]
    fn can_indent_second_item() {
        let model = cm("<ul><li>First item</li><li>{Second item}|</li></ul>");
        assert!(
            model.can_indent_list_item_handle(&DomHandle::from_raw(vec![0, 1]))
        );
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
        model.indent_list_item_handles(&vec![DomHandle::from_raw(vec![0, 1])]);
        assert_eq!(tx(&model), "<ul><li>First item<ul><li>Second item</li></ul></li><li>Third item|</li></ul>");
    }

    #[test]
    fn indent_list_item_to_previous_works() {
        let mut model = cm("<ul><li>First item<ul><li>Second item</li></ul></li><li>Third item|</li></ul>");
        model.indent_list_item_handles(&vec![DomHandle::from_raw(vec![0, 1])]);
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
