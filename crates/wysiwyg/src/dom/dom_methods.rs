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

//! Methods on Dom that modify its contents and are guaranteed to conform to
//! our invariants e.g. no empty text nodes, no adjacent text nodes.

use crate::dom::nodes::ContainerNodeKind;
use crate::dom::unicode_string::UnicodeStr;
use crate::{DomHandle, DomNode, ToHtml, UnicodeString};

use super::action_list::{DomAction, DomActionList};
use super::nodes::dom_node::DomNodeKind;
use super::nodes::{ContainerNode, TextNode};
use super::unicode_string::{UnicodeStrExt, UnicodeStringExt};
use super::{Dom, Range};

impl<S> Dom<S>
where
    S: UnicodeString,
{
    /// Removes node at given handle from the dom, and if it has children
    /// moves them to its parent container children.
    pub fn remove_and_keep_children(&mut self, node_handle: &DomHandle) {
        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();

        let parent = self.parent_mut(node_handle);
        let index = node_handle.index_in_parent();
        let node = parent.remove_child(index);
        let mut last_index = index;
        if let DomNode::Container(mut node) = node {
            for i in (0..node.children().len()).rev() {
                let child = node.remove_child(i);
                parent.insert_child(index, child);
                last_index += 1;
            }
        }

        // Clean up any adjacent text nodes
        merge_if_adjacent_text_nodes(parent, last_index - 1);
        if index > 0 {
            merge_if_adjacent_text_nodes(parent, index - 1);
        }

        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();
    }

    pub fn replace_text_in(&mut self, new_text: S, start: usize, end: usize) {
        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();

        let range = self.find_range(start, end);
        let deleted_handles = if range.is_empty() {
            if !new_text.is_empty() {
                self.append_at_end_of_document(DomNode::new_text(new_text));
            }
            Vec::new()
        // We check for the first starting_link_handle if any
        // Because for links we always add the text to the next sibling
        } else if let Some(starting_link_handle) =
            first_shrinkable_link_node_handle(&range)
        {
            // We replace and delete as normal with an empty string on the current range
            let deleted_handles =
                self.replace_multiple_nodes(&range, "".into());
            // Then we set the new text value in the next sibling node (or create a new one if none exists)
            self.set_new_text_in_next_sibling_node(
                starting_link_handle,
                new_text,
            );
            deleted_handles
        } else {
            self.replace_multiple_nodes(&range, new_text)
        };

        self.merge_adjacent_text_nodes_after_replace(range, deleted_handles);

        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();
    }

    /// Deletes the given [to_delete] nodes and then removes any given parent nodes that became
    /// empty, recursively.
    /// Returns a list of all the nodes that were deleted
    /// TODO: This function does not preserve invariants - the functions that
    /// call it should be moved into dom_methods, and they should grow
    /// assertions at the beginning and end that they preserve invariants,
    /// then this function can become non-public. (All public methods in
    /// dom_methods should preserve invariants.)
    pub(crate) fn delete_nodes(
        &mut self,
        mut to_delete: Vec<DomHandle>,
    ) -> Vec<DomHandle> {
        let mut deleted = Vec::new();

        // Delete in reverse order to avoid invalidating handles
        to_delete.reverse();

        // We repeatedly delete to ensure anything that became empty because
        // of deletions is itself deleted.
        while !to_delete.is_empty() {
            // Keep a list of things we will delete next time around the loop
            let mut new_to_delete = Vec::new();

            for handle in to_delete.into_iter().filter(not_root) {
                let index_in_parent = handle.index_in_parent();
                let parent = self.parent_mut(&handle);
                parent.remove_child(index_in_parent);
                adjust_handles_for_delete(&mut new_to_delete, &handle);
                deleted.push(handle);
                if parent.children().is_empty() {
                    new_to_delete.push(parent.handle());
                }
            }

            to_delete = new_to_delete;
        }
        deleted
    }

    fn set_new_text_in_next_sibling_node(
        &mut self,
        node_handle: DomHandle,
        new_text: S,
    ) {
        if let Some(sibling_text_node) =
            self.first_next_sibling_text_node_mut(&node_handle)
        {
            let mut data = sibling_text_node.data().to_owned();
            data.insert(0, &new_text);
            sibling_text_node.set_data(data);
        } else if !new_text.is_empty() {
            let new_child = DomNode::new_text(new_text);
            let parent = self.parent_mut(&node_handle);
            let index = node_handle.index_in_parent() + 1;
            parent.insert_child(index, new_child);
        }
    }

    fn first_next_sibling_text_node_mut(
        &mut self,
        node_handle: &DomHandle,
    ) -> Option<&mut TextNode<S>> {
        let parent = self.parent(node_handle);
        let children_number = parent.children().len();
        if node_handle.index_in_parent() < children_number - 1 {
            let sibling = self.lookup_node_mut(&node_handle.next_sibling());
            let DomNode::Text(sibling_text_node) = sibling else {
                return None
            };
            Some(sibling_text_node)
        } else {
            None
        }
    }

    /// Returns a list of handles to all the nodes that we deleted
    fn replace_multiple_nodes(
        &mut self,
        range: &Range,
        new_text: S,
    ) -> Vec<DomHandle> {
        let len = new_text.len();
        let action_list = self.replace_in_text_nodes(range.clone(), new_text);

        let (to_add, to_delete, _) = action_list.grouped();
        let to_delete: Vec<DomHandle> =
            to_delete.into_iter().map(|a| a.handle).collect();

        // We only add nodes in one special case: when the selection ends at
        // a BR tag. In that case, the only nodes that might be deleted are
        // going to be before the one we add here, so their handles won't be
        // invalidated by the add we do here.
        for add_action in to_add.into_iter().rev() {
            let parent_handle = &add_action.parent_handle;
            let parent = self.lookup_node_mut(parent_handle);
            if let DomNode::Container(parent) = parent {
                parent.insert_child(add_action.index, add_action.node);
            } else {
                panic!("Parent was not a container!");
            }
        }

        // Delete the nodes marked for deletion
        let deleted_handles = if !to_delete.is_empty() {
            self.delete_nodes(to_delete)
        } else {
            Vec::new()
        };

        // If our range has length and covered multiple text-like nodes, join together
        // the two sides of the range.
        if range.start() != range.end() && range.leaves().count() > 1 {
            // join_nodes will use the first location of our range, so we must
            // check whether we deleted it!
            if let Some(first_loc) = range.locations.first() {
                if !deleted_handles.contains(&first_loc.node_handle) {
                    // Calculate the position 1 code unit after the end of the
                    // range, after the in-between characters have been
                    // deleted, and the new characters have been inserted.
                    let new_pos = range.start() + len + 1;

                    // join_nodes only requires that the first location in
                    // the supplied range has a valid handle.
                    // We think it's OK to pass in a range where later
                    // locations have been deleted.
                    // TODO: can we just pass in this handle, to avoid the
                    // ambiguity here?
                    self.join_nodes(range, new_pos);
                }
            }
        } else if let Some(first_leave) = range.leaves().next() {
            if let Some(ancestor_handle) = self
                .find_first_non_matching_ancestor_in(
                    &deleted_handles,
                    &first_leave.node_handle,
                )
            {
                self.join_nodes_in_container(&ancestor_handle);
            }
        }
        deleted_handles
    }

    fn find_first_non_matching_ancestor_in(
        &self,
        list: &Vec<DomHandle>,
        node_handle: &DomHandle,
    ) -> Option<DomHandle> {
        fn parent_handle_in_list(
            list: &Vec<DomHandle>,
            handle: &DomHandle,
        ) -> Option<DomHandle> {
            if handle.has_parent() {
                let parent_handle = handle.parent_handle();
                if !list.contains(&parent_handle) {
                    Some(parent_handle)
                } else {
                    parent_handle_in_list(list, &parent_handle)
                }
            } else {
                None
            }
        }
        parent_handle_in_list(list, node_handle)
    }

    pub(crate) fn join_nodes_in_container(
        &mut self,
        container_handle: &DomHandle,
    ) {
        let child_count = if let DomNode::Container(container) =
            self.lookup_node(container_handle)
        {
            container.children().len()
        } else {
            panic!("Parent node should be a container");
        };

        if child_count > 0 {
            for i in (0..child_count - 1).rev() {
                let handle = container_handle.child_handle(i);
                let next_handle = container_handle.child_handle(i + 1);
                let next_node = self.lookup_node(&next_handle);
                let node = self.lookup_node(&handle);

                if node.can_push(next_node) {
                    let mut next_node = self.remove(&next_handle);
                    let node_mut = self.lookup_node_mut(&handle);
                    node_mut.push(&mut next_node);
                }
            }
        }
    }

    /// Given a range to replace and some new text, modify the nodes in the
    /// range to replace the text with the supplied text.
    /// Returns a list of actions to be done to the Dom (add or remove nodes).
    /// NOTE: all nodes to be created are later in the Dom than all nodes to
    /// be deleted, so you can safely add them before performing the
    /// deletions, and the handles of the deletions will remain valid.
    fn replace_in_text_nodes(
        &mut self,
        range: Range,
        new_text: S,
    ) -> DomActionList<S> {
        let mut action_list = DomActionList::default();
        let mut first_text_node = true;

        let start = range.start();
        let end = range.end();

        for loc in range.into_iter() {
            let mut node = self.lookup_node_mut(&loc.node_handle);
            match &mut node {
                DomNode::Container(_) => {
                    // Nothing to do for container nodes
                }
                DomNode::LineBreak(_) => {
                    match (loc.start_offset, loc.end_offset) {
                        (0, 1) => {
                            // Whole line break is selected, delete it
                            action_list.push(DomAction::remove_node(
                                loc.node_handle.clone(),
                            ));
                        }
                        (1, 1) => {
                            // Cursor is after line break, no need to delete
                        }
                        (0, 0) => {
                            if !new_text.is_empty() {
                                let node =
                                    DomNode::new_text(new_text.clone());
                                action_list.push(DomAction::add_node(
                                    loc.node_handle.parent_handle(),
                                    loc.node_handle.index_in_parent(),
                                    node,
                                ));
                            }
                        }
                        _ => panic!(
                            "Tried to insert text into a line break with offset != 0 or 1. \
                            Start offset: {}, end offset: {}",
                            loc.start_offset,
                            loc.end_offset,
                        ),
                    }
                    if start >= loc.position && end == loc.position + 1 {
                        // NOTE: if you add something else to `action_list` you will
                        // probably break our assumptions in the method that
                        // calls this one!
                        // We are assuming we only add nodes AFTER all the
                        // deleted nodes. (That is true in this case, because
                        // we are checking that the selection ends inside this
                        // line break.)
                        if !new_text.is_empty() {
                            action_list.push(DomAction::add_node(
                                loc.node_handle.parent_handle(),
                                loc.node_handle.index_in_parent() + 1,
                                DomNode::new_text(new_text.clone()),
                            ));
                        }
                    }
                }
                DomNode::Text(node) => {
                    let old_data = node.data();

                    // If this is not the first node, and the selections spans
                    // it, delete it.
                    if loc.start_offset == 0
                        && loc.end_offset == old_data.len()
                        && !first_text_node
                    {
                        action_list
                            .push(DomAction::remove_node(loc.node_handle));
                    } else {
                        // Otherwise, delete the selected text
                        let mut new_data =
                            old_data[..loc.start_offset].to_owned();

                        // and replace with the new content
                        if first_text_node {
                            new_data.push(new_text.deref());
                        }

                        new_data.push(&old_data[loc.end_offset..]);
                        if new_data.is_empty() {
                            action_list
                                .push(DomAction::remove_node(loc.node_handle));
                        } else {
                            node.set_data(new_data);
                        }
                    }

                    first_text_node = false;
                }
                DomNode::Zwsp(_) => todo!(),
            }
        }
        action_list
    }

    fn merge_adjacent_text_nodes_after_replace(
        &mut self,
        replaced_range: Range,
        deleted_handles: Vec<DomHandle>,
    ) {
        // If we've ended up with adjacent text nodes, merge them
        if let Some(first_location) = replaced_range.locations.first() {
            let first_handle = &first_location.node_handle;
            if deleted_handles.contains(first_handle) {
                // If we deleted the first node in the range ...
                if first_handle.index_in_parent() > 0 {
                    // ... and that was not the first in its parent,
                    // then merge the node before with the next.
                    let prev_handle = first_handle.prev_sibling();
                    self.merge_text_nodes_around(&prev_handle);
                }
            } else {
                // If the first node of the range still exists, then
                // merge it with the next, and potentially also the
                // previous.
                self.merge_text_nodes_around(&first_location.node_handle);
            }
        }
    }

    fn merge_text_nodes_around(&mut self, handle: &DomHandle) {
        // TODO: make this method not public because it is used to make
        // the invariants true, instead of assuming they are true at the
        // beginning!
        // Instead, move another method into here, and make it call this one.

        let parent = self.parent_mut(handle);
        let idx = handle.index_in_parent();
        if idx > 0 {
            merge_if_adjacent_text_nodes(parent, idx - 1);
        }
        merge_if_adjacent_text_nodes(parent, idx);

        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();
    }

    pub(crate) fn split_sub_tree(
        &mut self,
        from_handle: &DomHandle,
        offset: usize,
        depth: usize,
        up_to_handle: Option<DomHandle>,
    ) -> DomNode<S> {
        // Create new 'root' node to contain the split sub-tree
        let mut new_subtree = DomNode::Container(ContainerNode::new(
            S::default(),
            ContainerNodeKind::Generic,
            None,
            Vec::new(),
        ));
        new_subtree.set_handle(DomHandle::root());

        let path = from_handle.sub_handle_up_to(depth).raw().clone();
        self.split_sub_tree_at(
            path,
            from_handle.raw()[depth],
            depth,
            offset,
            from_handle,
            up_to_handle,
            &mut new_subtree,
        );

        let cur_html = self.to_html().to_string();
        let subtree_html = new_subtree.to_html().to_string();

        let keep_empty_list_items =
            new_subtree.iter_subtree().find(|n| n.is_list()).is_none();
        self.remove_empty_container_nodes(keep_empty_list_items);

        new_subtree
    }

    fn split_sub_tree_at<'a>(
        &'a mut self,
        path: Vec<usize>,
        index_in_parent: usize,
        min_depth: usize,
        offset: usize,
        handle: &DomHandle,
        up_to_handle: Option<DomHandle>,
        result: &'a mut DomNode<S>,
    ) {
        let mut path = path;
        let cur_handle = DomHandle::from_raw(path.clone());
        path.push(index_in_parent);
        let mut has_next_level = false;
        let cur_subtree = self.lookup_node_mut(&cur_handle);
        let mut removed_nodes = Vec::new();
        if let DomNode::Container(container) = cur_subtree {
            let max_index = if let Some(up_to_handle) = up_to_handle.clone() {
                if up_to_handle.has_parent()
                    && up_to_handle.parent_handle() == cur_handle
                {
                    up_to_handle.index_in_parent()
                } else {
                    container.children().len()
                }
            } else {
                container.children().len()
            };
            for idx in (index_in_parent..max_index).rev() {
                if idx == index_in_parent {
                    let child = container.get_child_mut(idx).unwrap();
                    match child {
                        DomNode::Container(c) => {
                            has_next_level = true;
                            removed_nodes.insert(
                                0,
                                DomNode::Container(
                                    c.clone_with_new_children(Vec::new()),
                                ),
                            );
                        }
                        DomNode::Text(text_node) => {
                            if offset == 0 {
                                removed_nodes
                                    .insert(0, container.remove_child(idx));
                            } else if offset >= text_node.data().chars().count()
                            {
                                // Do nothing
                            } else {
                                let left_data =
                                    text_node.data()[..offset].to_owned();
                                let right_data =
                                    text_node.data()[offset..].to_owned();
                                text_node.set_data(left_data);
                                removed_nodes
                                    .insert(0, DomNode::new_text(right_data));
                            }
                        }
                        _ => {
                            removed_nodes.insert(0, child.clone());
                        }
                    }
                } else {
                    removed_nodes.insert(0, container.remove_child(idx));
                }
            }
        }
        let mut new_subtree_at_prev_level = result;
        if (path.len() - min_depth) > 1 {
            if let DomNode::Container(c) = new_subtree_at_prev_level {
                new_subtree_at_prev_level = c.get_child_mut(0).unwrap();
            }
        }
        if let DomNode::Container(new_subtree) = new_subtree_at_prev_level {
            if !removed_nodes.is_empty() {
                for node in removed_nodes {
                    new_subtree.append_child(node);
                }
            }
        }

        if has_next_level {
            let index_at_level = if path.len() < handle.depth() {
                handle.raw()[path.len()]
            } else {
                0
            };
            self.split_sub_tree_at(
                path,
                index_at_level,
                min_depth,
                offset,
                handle,
                up_to_handle.clone(),
                new_subtree_at_prev_level,
            );
        }
    }

    fn remove_empty_container_nodes(&mut self, keep_empty_list_items: bool) {
        let last_handle_in_dom = self.last_node_handle();
        let handles_in_reverse: Vec<DomHandle> =
            self.handle_iter_from(&last_handle_in_dom).rev().collect();
        for handle in handles_in_reverse {
            let mut needs_removal = false;
            if let DomNode::Container(container) = self.lookup_node(&handle) {
                if container.children().is_empty()
                    && !(keep_empty_list_items && container.is_list_item())
                {
                    needs_removal = true;
                }
            }
            if needs_removal && !handle.is_root() {
                self.remove(&handle);
            }
        }
    }
}

/// Look at the children of parent at index and index + 1. If they are both
/// text nodes, merge them into the first and delete the second.
/// If either child does not exist, do nothing.
fn merge_if_adjacent_text_nodes<S>(parent: &mut ContainerNode<S>, index: usize)
where
    S: UnicodeString,
{
    let previous_child = parent.children().get(index);
    let after_child = parent.children().get(index + 1);
    if let (Some(DomNode::Text(t1)), Some(DomNode::Text(t2))) =
        (previous_child, after_child)
    {
        let mut data = t1.data().to_owned();
        data.push(t2.data());
        if let Some(DomNode::Text(t1_mut)) = parent.get_child_mut(index) {
            t1_mut.set_data(data);
            parent.remove_child(index + 1);
        } else {
            unreachable!("t1 was a text node but t1_mut was not!");
        }
    }
}

fn first_shrinkable_link_node_handle(range: &Range) -> Option<DomHandle> {
    let Some(link_loc) = range.locations.iter().find(|loc| {
            loc.kind == DomNodeKind::Link && !loc.is_covered() && loc.is_start()
        }) else {
            return None
        };
    Some(link_loc.node_handle.clone())
}

fn not_root(handle: &DomHandle) -> bool {
    !handle.is_root()
}

fn adjust_handles_for_delete(
    handles: &mut Vec<DomHandle>,
    deleted: &DomHandle,
) {
    let mut indices_in_handles_to_delete = Vec::new();
    let mut handles_to_replace = Vec::new();

    let parent = deleted.parent_handle();
    for (i, handle) in handles.iter().enumerate() {
        if deleted.is_ancestor_of(handle) {
            // We are the deleted node (or a descendant of it)
            indices_in_handles_to_delete.push(i);
        } else if parent.is_ancestor_of(handle) {
            // We are a sibling of the deleted node (or a descendant of one)

            // If we're after a deleted node, reduce our index
            let mut child_index = handle.raw()[parent.raw().len()];
            let deleted_index = *deleted.raw().last().unwrap();
            if child_index > deleted_index {
                child_index -= 1;
            }

            // Create a handle with the adjusted index (but missing anything
            // after the delete node's length).
            let mut new_handle = parent.child_handle(child_index);

            // Add back the rest of our original handle, unadjusted
            for h in &handle.raw()[deleted.raw().len()..] {
                new_handle = new_handle.child_handle(*h);
            }
            handles_to_replace.push((i, new_handle));
        }
    }

    for (i, new_handle) in handles_to_replace {
        handles[i] = new_handle;
    }

    indices_in_handles_to_delete.reverse();
    for i in indices_in_handles_to_delete {
        handles.remove(i);
    }
}

#[cfg(test)]
mod test {
    use crate::dom::DomHandle;
    use crate::tests::testutils_composer_model::{cm, tx};
    use crate::ToHtml;

    use super::*;

    #[test]
    fn can_adjust_handles_when_removing_nodes() {
        let mut handles = vec![
            DomHandle::from_raw(vec![1, 2, 3]), // Ignored because before
            DomHandle::from_raw(vec![2, 3, 4, 5]), // Deleted because inside
            DomHandle::from_raw(vec![3, 4, 5]), // Adjusted because after
            DomHandle::from_raw(vec![3]),       // Adjusted because after
        ];

        let to_delete = DomHandle::from_raw(vec![2]);

        adjust_handles_for_delete(&mut handles, &to_delete);

        assert_eq!(*handles[0].raw(), vec![1, 2, 3]);
        assert_eq!(*handles[1].raw(), vec![2, 4, 5]);
        assert_eq!(*handles[2].raw(), vec![2]);
        assert_eq!(handles.len(), 3);
    }

    #[test]
    fn can_adjust_handles_when_removing_nested_nodes() {
        let mut handles = vec![
            DomHandle::from_raw(vec![0, 9, 1, 2, 3]),
            DomHandle::from_raw(vec![0, 9, 2, 3, 4, 5]),
            DomHandle::from_raw(vec![0, 9, 3, 4, 5]),
            DomHandle::from_raw(vec![0, 9, 3]),
        ];

        let to_delete = DomHandle::from_raw(vec![0, 9, 2]);

        adjust_handles_for_delete(&mut handles, &to_delete);

        assert_eq!(*handles[0].raw(), vec![0, 9, 1, 2, 3]);
        assert_eq!(*handles[1].raw(), vec![0, 9, 2, 4, 5]);
        assert_eq!(*handles[2].raw(), vec![0, 9, 2]);
        assert_eq!(handles.len(), 3);
    }

    #[test]
    fn delete_nodes_refuses_to_delete_root() {
        let mut model = cm("|");
        model
            .state
            .dom
            .delete_nodes(vec![model.state.dom.document_handle()]);
        assert_eq!(tx(&model), "|")
    }

    #[test]
    fn delete_nodes_refuses_recursively_to_delete_root() {
        let mut model = cm("a|");
        model
            .state
            .dom
            .delete_nodes(vec![DomHandle::from_raw(vec![0])]);
        assert_eq!(tx(&model), "|")
    }

    #[test]
    fn split_dom_simple() {
        let mut model = cm("Text|<b>bold</b><i>italic</i>");
        let ret = model.state.dom.split_sub_tree(
            &DomHandle::from_raw(vec![1, 0]),
            2,
            0,
            None,
        );
        assert_eq!(ret.to_html().to_string(), "<b>ld</b><i>italic</i>");
    }

    #[test]
    fn split_dom_with_nested_formatting() {
        let mut model = cm("<u>Text|<b>bold</b><i>italic</i></u>");
        let ret = model.state.dom.split_sub_tree(
            &DomHandle::from_raw(vec![0, 1, 0]),
            2,
            0,
            None,
        );
        assert_eq!(ret.to_html().to_string(), "<u><b>ld</b><i>italic</i></u>");
    }

    #[test]
    fn split_dom_with_nested_formatting_at_sub_level() {
        let mut model = cm("<u>Text|<b>bold</b><i>italic</i></u>");
        let ret = model.state.dom.split_sub_tree(
            &DomHandle::from_raw(vec![0, 1, 0]),
            2,
            1,
            None,
        );
        assert_eq!(ret.to_html().to_string(), "<b>ld</b><i>italic</i>")
    }

    #[test]
    fn split_dom_with_lists() {
        let mut model =
            cm("<ul><li>Text|</li><li><b>bold</b><i>italic</i></li></ul>");
        let depth = 0;
        let offset = 2;
        let ret = model.state.dom.split_sub_tree(
            &DomHandle::from_raw(vec![0, 1, 0, 0]),
            offset,
            depth,
            None,
        );
        assert_eq!(
            ret.to_html().to_string(),
            "<ul><li><b>ld</b><i>italic</i></li></ul>"
        )
    }

    #[test]
    fn split_dom_with_lists_at_sub_level() {
        let mut model =
            cm("<ul><li>Text|</li><li><b>bold</b><i>italic</i></li></ul>");
        let depth = 1;
        let offset = 2;
        let ret = model.state.dom.split_sub_tree(
            &DomHandle::from_raw(vec![0, 1, 0, 0]),
            offset,
            depth,
            None,
        );
        assert_eq!(ret.to_html().to_string(), "<li><b>ld</b><i>italic</i></li>")
    }

    #[test]
    fn split_dom_with_partial_handle() {
        let mut model = cm("<u>Text|<b>bold</b><i>italic</i></u>");
        let ret = model.state.dom.split_sub_tree(
            &DomHandle::from_raw(vec![0, 1]),
            2,
            0,
            None,
        );
        assert_eq!(ret.to_html().to_string(), "<u><b>ld</b><i>italic</i></u>");
    }
}
