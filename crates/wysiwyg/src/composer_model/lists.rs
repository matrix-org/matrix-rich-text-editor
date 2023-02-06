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
use crate::dom::nodes::dom_node::DomNodeKind::Paragraph;
use crate::dom::nodes::{ContainerNode, DomNode};
use crate::dom::range::DomLocationPosition;
use crate::dom::range::DomLocationPosition::Before;
use crate::dom::{Dom, DomHandle, DomLocation, Range};
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
        let top_most_locations =
            self.find_top_most_list_item_locations(&range.locations);
        if !top_most_locations.is_empty()
            && self.can_indent(&top_most_locations)
        {
            self.push_state_to_history();
            self.indent_locations(&top_most_locations);
            self.create_update_replace_all()
        } else {
            ComposerUpdate::keep()
        }
    }

    pub fn unindent(&mut self) -> ComposerUpdate<S> {
        // push_state_to_history is called if we can unindent
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let top_most_locations =
            self.find_top_most_list_item_locations(&range.locations);
        if self.can_unindent(&top_most_locations) {
            self.push_state_to_history();
            self.unindent_locations(&top_most_locations);
            self.create_update_replace_all()
        } else {
            ComposerUpdate::keep()
        }
    }

    pub fn can_indent(&self, locations: &[DomLocation]) -> bool {
        let list_item_locations: Vec<&DomLocation> = locations
            .iter()
            .filter(|l| l.kind == DomNodeKind::ListItem)
            .collect();
        if list_item_locations.is_empty() {
            return false;
        }
        let mut can_indent = true;
        for loc in list_item_locations {
            if loc.relative_position() == Before {
                continue;
            }
            can_indent &= self.can_indent_list_item_handle(&loc.node_handle);
        }
        can_indent
    }

    pub fn can_unindent(&self, locations: &[DomLocation]) -> bool {
        let list_item_locations: Vec<&DomLocation> = locations
            .iter()
            .filter(|l| l.kind == DomNodeKind::ListItem)
            .collect();
        if list_item_locations.is_empty() {
            return false;
        }
        let mut can_unindent = true;
        for loc in list_item_locations {
            if loc.relative_position() == Before {
                continue;
            }
            can_unindent &= self.can_unindent_handle(&loc.node_handle)
        }
        can_unindent
    }

    pub(crate) fn find_top_most_list_item_locations(
        &self,
        locations: &[DomLocation],
    ) -> Vec<DomLocation> {
        // Find any selected leaves and block nodes
        let leaves_and_empty_block_nodes = locations.iter().filter(|l| {
            l.is_leaf() || (l.kind.is_block_kind() && l.is_empty())
        });
        // Gather their ancestor list items, if any
        let list_item_handles: Vec<DomHandle> = leaves_and_empty_block_nodes
            .into_iter()
            .filter_map(|l| {
                self.state
                    .dom
                    .find_ancestor_list_item_or_self(&l.node_handle)
            })
            .collect();
        // Find what is the top most level of all of those list handles
        let top_most_level = list_item_handles
            .iter()
            .map(|h| h.depth())
            .min()
            .unwrap_or(0);
        // Retrieve the actual list item locations
        let top_most_list_items: Vec<DomLocation> = locations
            .iter()
            .filter(|l| {
                l.kind == DomNodeKind::ListItem
                    && l.node_handle.depth() == top_most_level
                    && l.relative_position() != Before
            })
            .cloned()
            .collect();
        top_most_list_items
    }

    pub(crate) fn do_backspace_in_list(
        &mut self,
        list_item_handle: &DomHandle,
    ) -> ComposerUpdate<S> {
        let list_item_node = self.state.dom.lookup_node(list_item_handle);
        let list_node_handle = list_item_node.handle().parent_handle();
        if let DomNode::Container(list_item) = list_item_node {
            if list_item.has_no_text()
                || list_item_handle.index_in_parent() == 0
            {
                self.state.dom.extract_list_items(
                    &list_node_handle,
                    list_item_handle.index_in_parent(),
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
            if let Some(block_location) = range.deepest_block_node(None) {
                self.single_leaf_list_toggle(
                    list_type,
                    &block_location.node_handle,
                )
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
            self.state.dom.find_ancestor_list_item_or_self(handle);
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
        let nodes_iterator =
            if range.has_single_top_level_node(Some(DomNodeKind::Quote)) {
                // Single top level quote means we're trying to create a list inside
                // the quote, we should wrap the nodes from one depth further into it.
                range.locations_at_depth(range.top_level_depth() + 1)
            } else {
                range.locations_at_depth(range.top_level_depth())
            };

        let handles = nodes_iterator.map(|l| &l.node_handle).collect();
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
        let node = self.state.dom.lookup_node(handle);
        if node.is_list_item() {
            handle.index_in_parent() > 0
        } else {
            false
        }
    }

    pub(crate) fn can_unindent_handle(&self, handle: &DomHandle) -> bool {
        // Check that there are at least 2 ancestor lists
        if let Some(closest_list_handle) =
            self.find_closest_ancestor_of_kind(handle, DomNodeKind::List)
        {
            self.find_closest_ancestor_of_kind(
                &closest_list_handle,
                DomNodeKind::List,
            )
            .is_some()
        } else {
            false
        }
    }

    fn indent_locations(&mut self, locations: &[DomLocation]) {
        let handles: Vec<DomHandle> =
            locations.iter().map(|l| l.node_handle.clone()).collect();
        self.indent_list_item_handles(&handles);
    }

    fn indent_list_item_handles(&mut self, handles: &Vec<DomHandle>) {
        // Pre-checks
        if handles.is_empty() {
            return;
        }
        let can_indent =
            handles.iter().all(|h| self.can_indent_list_item_handle(h));
        if !can_indent {
            return;
        }

        let parent_handle = handles[0].parent_handle();
        let all_have_same_parent =
            handles.iter().all(|h| h.parent_handle() == parent_handle);
        if !all_have_same_parent {
            return;
        }

        // Sort handles to avoid issues where we delete a former handle so the rest become invalid
        let mut sorted_handles = handles.clone();
        sorted_handles.sort();

        let first_handle = sorted_handles.get(0).unwrap();
        let insert_into_handle = first_handle.prev_sibling();

        let parent_list_type = self
            .state
            .dom
            .parent(first_handle)
            .get_list_type()
            .unwrap()
            .clone();

        // Remove ListItems to indent from the parent List
        let mut removed_list_items = Vec::new();
        for handle in sorted_handles.iter().rev() {
            removed_list_items.insert(0, self.state.dom.remove(handle));
        }

        if let DomNode::Container(dest_list_item) =
            &mut self.state.dom.lookup_node_mut(&insert_into_handle)
        {
            // Wrap any existing inline nodes inside the destination ListItem into a paragraph
            if dest_list_item.children().len() == 1
                && !dest_list_item.get_child(0).unwrap().is_block_node()
            {
                let children = dest_list_item.remove_children();
                let paragraph = DomNode::new_paragraph(children);
                dest_list_item.append_child(paragraph);
            }

            // Then add a new list with the removed ListItems
            dest_list_item.append_child(DomNode::new_list(
                parent_list_type,
                removed_list_items,
            ));
        } else {
            panic!("Destination list item must be a container");
        }
        // We'll join adjacent Lists, so even if we appended a new List above, this would be the
        // same as pushing new ListItems to that List
        self.state.dom.join_nodes_in_container(&insert_into_handle);
    }

    fn unindent_locations(&mut self, locations: &[DomLocation]) {
        let handles: Vec<DomHandle> =
            locations.iter().map(|l| l.node_handle.clone()).collect();
        self.unindent_handles(&handles);
    }

    fn unindent_handles(&mut self, handles: &Vec<DomHandle>) {
        /// Helper to get a container node without so many unwraps
        fn get_container<'a, S: UnicodeString>(
            dom: &'a Dom<S>,
            handle: &DomHandle,
        ) -> &'a ContainerNode<S> {
            dom.lookup_node(handle).as_container().unwrap()
        }
        // Pre-checks
        if handles.is_empty() {
            return;
        }
        let can_unindent = handles.iter().all(|h| self.can_unindent_handle(h));
        if !can_unindent {
            return;
        }

        let first_handle = handles[0].clone();
        let parent_handle = first_handle.parent_handle();
        let all_have_same_parent =
            handles.iter().all(|h| h.parent_handle() == parent_handle);
        if !all_have_same_parent {
            return;
        }

        // Sort handles to avoid issues where we delete a former handle so the rest become invalid
        let mut sorted_handles = handles.clone();
        sorted_handles.sort();

        // We should always insert the new List inside the next ListItem sibling
        let insert_into_handle = parent_handle.parent_handle().next_sibling();

        // Remove the selected ListItems
        let mut removed_list_items = Vec::new();
        for handle in sorted_handles.iter().rev() {
            removed_list_items.insert(0, self.state.dom.remove(handle));
        }

        let list_type = get_container(&self.state.dom, &parent_handle)
            .get_list_type()
            .unwrap()
            .clone();
        let remaining_list_child_count =
            get_container(&self.state.dom, &parent_handle)
                .children()
                .len();

        // Remove any remaining ListItems after the removed ones to add them to a new List child.
        // The ListItems before the removed ones will be kept in the same List child.
        let mut list_items_after_removed_ones = Vec::new();
        for i in
            (first_handle.index_in_parent()..remaining_list_child_count).rev()
        {
            let child = self.state.dom.remove(&parent_handle.child_handle(i));
            list_items_after_removed_ones.insert(0, child);
        }

        // FIXME: extract transaction to a dom method if possible
        #[cfg(any(test, feature = "assert-invariants"))]
        self.state.dom.start_transaction();

        let list_became_empty =
            get_container(&self.state.dom, &parent_handle).is_empty();
        if list_became_empty {
            // If List containing the selected ListItems became empty, remove it
            self.state.dom.remove(&parent_handle);
        }

        if !list_items_after_removed_ones.is_empty() {
            // If the internal List didn't become empty, we need to move the remaining
            // second half of it into a List inside last removed ListItem.
            // Example case:
            //
            // - First
            //     - Second
            //     - {Third}|
            //     - Fourth
            //
            //  Becomes:
            //
            //  - First
            //     - Second
            //  - {Third}|
            //     - Fourth
            //
            // 'Fourth' here would be inside `list_items_after_removed_ones` and will be added
            // to the last un-indented ListItem, 'Third'.
            let mut last_removed_list_item = removed_list_items.pop().unwrap();
            let container = last_removed_list_item.as_container_mut().unwrap();
            let needs_paragraph =
                container.children().iter().any(|n| !n.is_block_node());
            if needs_paragraph {
                let children = container.remove_children();
                let paragraph = DomNode::new_paragraph(children);
                container.append_child(paragraph);
            }
            let new_list =
                DomNode::new_list(list_type, list_items_after_removed_ones);
            container.append_child(new_list);
            removed_list_items.push(last_removed_list_item);
        }

        // Unwrap existing paragraph in the parent ListItem if needed
        {
            let orig_parent_list_item =
                get_container(&self.state.dom, &parent_handle.parent_handle());
            // If only 1 node is left and it's a paragraph, unwrap its children
            if orig_parent_list_item.children().len() == 1
                && orig_parent_list_item.children()[0].kind() == Paragraph
            {
                self.state.dom.remove_and_keep_children(
                    &orig_parent_list_item.handle().child_handle(0),
                );
            }
        }

        // Insert the removed ListItems into the next sibling of the parent ListItem
        self.state
            .dom
            .insert(&insert_into_handle, removed_list_items);

        self.state.dom.join_nodes_in_container(&insert_into_handle);

        #[cfg(any(test, feature = "assert-invariants"))]
        self.state.dom.end_transaction();
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
        assert_eq!(tx(&model), "<ul><li><p>First item</p><ul><li>Second item</li></ul></li><li>Third item|</li></ul>");
    }

    #[test]
    fn indent_list_item_to_previous_works() {
        let mut model = cm("<ul><li><p>First item</p><ul><li>Second item</li></ul></li><li>Third item|</li></ul>");
        model.indent_list_item_handles(&vec![DomHandle::from_raw(vec![0, 1])]);
        assert_eq!(tx(&model), "<ul><li><p>First item</p><ul><li>Second item</li><li>Third item|</li></ul></li></ul>");
    }

    #[test]
    fn can_unindent_handle_simple_case_works() {
        let model = cm("<ul><li>First item<ul><li>{Second item</li><li>Third item}|</li></ul></li></ul>");
        let handle = DomHandle::from_raw(vec![0, 0, 1, 0, 0]);
        assert!(model.can_unindent_handle(&handle));
    }

    #[test]
    fn can_unindent_simple_case_works() {
        let model = cm("<ul><li><p>First item</p><ul><li>{Second item</li><li>Third item}|</li></ul></li></ul>");
        let locations = get_range_locations(&model);
        let top_most_list_item_locations =
            model.find_top_most_list_item_locations(&locations);
        assert!(model.can_unindent(&top_most_list_item_locations));
    }

    #[test]
    fn can_unindent_with_only_one_list_level_fails() {
        let model = cm("<ul><li>First item</li><li>{Second item</li><li>Third item}|</li></ul>");
        let locations = get_range_locations(&model);
        let top_most_list_item_locations =
            model.find_top_most_list_item_locations(&locations);
        assert!(!model.can_unindent(&top_most_list_item_locations));
    }

    #[test]
    fn unindent_handles_simple_case_works() {
        let mut model =
            cm("<ul><li><p>First item</p><ul><li>{Second item}|</li></ul></li></ul>");
        let handles = vec![DomHandle::from_raw(vec![0, 0, 1, 0])];
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
