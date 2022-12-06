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

use crate::dom::action_list::{DomAction, DomActionList};
use crate::dom::nodes::{ContainerNodeKind, DomNode};
use crate::dom::unicode_string::UnicodeStringExt;
use crate::dom::{Dom, DomHandle, DomLocation, Range};
use crate::UnicodeString;

/// Handles joining together nodes after an edit event.
///
/// For example, if your selection starts in a bold tag, leaves that tag,
/// and then ends up in another bold tag, the final result should be a
/// single bold tag containing all your text.
impl<S> Dom<S>
where
    S: UnicodeString,
{
    /// After the selection range we were given in from_range has been deleted,
    /// join any nodes that match up across the selection.
    pub(crate) fn join_nodes(&mut self, range: &Range, new_pos: usize) {
        if let Some(start_handle) = self.first_text_handle(range) {
            self.join_structure_nodes(&start_handle, new_pos);
            self.join_format_nodes_at_index(new_pos);
        }
    }

    /// Join format node at [handle], if any, with its previous sibling if it's a compatible format
    /// node.
    /// The passed [action_list] is used in a special way here: instead of collecting actions to be
    /// performed in the future, we're using it to keep track of moved nodes and its handles.
    pub(crate) fn join_format_node_with_prev(
        &mut self,
        handle: &DomHandle,
        action_list: &mut DomActionList<S>,
    ) {
        self.join_format_nodes_at_level(handle, 0, action_list);
    }

    fn join_structure_nodes(
        &mut self,
        start_handle: &DomHandle,
        new_pos: usize,
    ) {
        // Find next node
        if let Some(mut next_handle) = self.find_leaf_containing(new_pos) {
            // Find struct parents
            if let (Some(struct_parent_start), Some(struct_parent_next)) = self
                .find_closest_structure_ancestors_to_join(
                    start_handle,
                    &next_handle,
                )
            {
                if struct_parent_start != struct_parent_next {
                    // Move children
                    let (new_index, _) = self.move_children_and_delete_parent(
                        &struct_parent_next,
                        &struct_parent_start,
                    );

                    next_handle = struct_parent_start.child_handle(new_index);
                }

                // Find ancestor lists
                let ancestors_start =
                    Self::list_of_ancestors_from_root(start_handle);
                let ancestors_next =
                    Self::list_of_ancestors_from_root(&next_handle);

                // Merge nodes based on ancestors
                self.do_join_structure_nodes(&ancestors_start, &ancestors_next);
            }
        }
    }

    fn join_format_nodes_at_index(&mut self, index: usize) {
        if let Some(next_node_handle) = self.find_leaf_containing(index) {
            self.join_format_node_with_prev(
                &next_node_handle,
                &mut DomActionList::default(),
            );
        }
    }

    fn join_format_nodes_at_level(
        &mut self,
        handle: &DomHandle,
        level: usize,
        action_list: &mut DomActionList<S>,
    ) {
        // Out of bounds
        if level >= handle.raw().len() {
            return;
        }
        let mut handle = handle.clone();
        // If the handle was moved, use updated value instead
        let moved_handle = action_list.find_moved_parent_or_self(&handle);
        if let Some((from_handle, to_handle)) = moved_handle {
            handle.replace_ancestor(from_handle, to_handle);
        }
        // Get the node handle at the current depth level
        let cur_handle = DomHandle::from_raw(handle.raw()[..=level].to_vec());
        let index_in_parent = if cur_handle.is_root() {
            0
        } else {
            cur_handle.index_in_parent()
        };
        // We're comparing it to the previous sibling, so there's no point in allowing index 0
        if index_in_parent > 0 {
            let prev_handle = cur_handle.prev_sibling();
            // Found a matching sibling node with the same format
            if self.can_merge_format_nodes(&prev_handle, &cur_handle) {
                // Move the contents from the current node to the previous one
                let (new_index, moved) = self
                    .move_children_and_delete_parent(&cur_handle, &prev_handle);
                // Next iteration
                let mut cur_path = handle.raw().clone();
                let prev_path = prev_handle.raw();
                cur_path[level] = prev_path[level];
                if level + 1 < cur_path.len() {
                    cur_path[level + 1] = new_index;
                }
                let new_handle = DomHandle::from_raw(cur_path);

                let move_actions: Vec<DomAction<S>> = moved
                    .into_iter()
                    .map(|(o, n)| DomAction::move_node(o, n))
                    .collect();
                action_list.extend(move_actions);
                self.join_format_nodes_at_level(
                    &new_handle,
                    level + 1,
                    action_list,
                );
            } else {
                // If both nodes couldn't be merged, try at the next level
                self.join_format_nodes_at_level(
                    &handle,
                    level + 1,
                    action_list,
                );
            }
        } else {
            // If there's no previous node, try at the next level
            self.join_format_nodes_at_level(&handle, level + 1, action_list);
        }
    }

    fn can_merge_format_nodes(
        &self,
        prev: &DomHandle,
        next: &DomHandle,
    ) -> bool {
        if let (DomNode::Container(prev_node), DomNode::Container(next_node)) =
            (self.lookup_node(prev), self.lookup_node(next))
        {
            if let (
                ContainerNodeKind::Formatting(prev_format),
                ContainerNodeKind::Formatting(next_format),
            ) = (prev_node.kind(), next_node.kind())
            {
                // Found a matching sibling node with the same format
                return prev_format == next_format;
            }
        }
        false
    }

    fn do_join_structure_nodes(
        &mut self,
        ancestors_start: &Vec<DomHandle>,
        ancestors_next: &Vec<DomHandle>,
    ) {
        let mut i = 0;
        let mut j = 0;
        while i < ancestors_start.len() && j < ancestors_next.len() {
            let start_handle = ancestors_start.get(i).unwrap();
            let next_handle = ancestors_next.get(j).unwrap();

            // If both lists contain this ancestor handle, continue to the next comparison.
            if start_handle == next_handle {
                i += 1;
                j += 1;
                continue;
            }

            let start_i = self.lookup_node(start_handle);
            let next_i = self.lookup_node(next_handle);

            match (start_i, next_i) {
                (DomNode::Container(start_i), DomNode::Container(next_i)) => {
                    // TODO: check if this is_structure_node verification is needed.
                    if start_i.is_structure_node()
                        && next_i.is_structure_node()
                        && start_i.name() == next_i.name()
                    {
                        // Both containers with the same tag.
                        // Move children from next to start node, remove next node.
                        let (new_index_in_parent, _) = self
                            .move_children_and_delete_parent(
                                next_handle,
                                start_handle,
                            );
                        // This alters ancestors, so we need to re-calculate them and start again.
                        let new_ancestors_next = Self::re_calculate_ancestors(
                            start_handle,
                            new_index_in_parent,
                            i,
                        );
                        // Restart the process
                        self.do_join_structure_nodes(
                            ancestors_start,
                            &new_ancestors_next,
                        );
                        return;
                    } else {
                        // Both containers, but different tags. We're done.
                        return;
                    }
                }
                (DomNode::Container(_), DomNode::Text(_)) => {
                    i += 1;
                }
                (DomNode::Text(start_i), DomNode::Text(next_i)) => {
                    let mut new_data = start_i.data().to_owned();
                    new_data.push(next_i.data());
                    let text_node = DomNode::new_text(new_data);
                    let old_parent = self.parent_mut(next_handle);
                    old_parent.remove_child(next_handle.index_in_parent());
                    let parent = self.parent_mut(start_handle);
                    parent.replace_child(
                        start_handle.index_in_parent(),
                        vec![text_node],
                    );
                    return;
                }
                _ => return,
            }
        }
    }

    fn re_calculate_ancestors(
        start_handle: &DomHandle,
        new_index_in_parent: usize,
        level: usize,
    ) -> Vec<DomHandle> {
        let mut new_next_path = start_handle.raw()[..level].to_vec();
        new_next_path.push(new_index_in_parent);
        let new_next_handle = DomHandle::from_raw(new_next_path);

        // Re-calculate ancestors
        Self::list_of_ancestors_from_root(&new_next_handle)
    }

    /// Given a position, find the text or line break node containing it
    fn find_leaf_containing(&self, pos: usize) -> Option<DomHandle> {
        let range = self.find_range(pos, pos);
        self.find_next_node_range(range)
    }

    fn find_next_node_range(&self, range: Range) -> Option<DomHandle> {
        range.leaves().next().map(|loc| loc.node_handle.clone())
    }

    /// Finds the closest structure ancestors to join for the start and end handles, or None if they
    /// can't be found.
    /// Note: If a ListItem is found, the parent List will be returned instead.
    fn find_closest_structure_ancestors_to_join(
        &self,
        start: &DomHandle,
        next: &DomHandle,
    ) -> (Option<DomHandle>, Option<DomHandle>) {
        let get_list_parent_if_is_list_item = |handle| {
            let node = self.lookup_node(&handle);
            if node.is_list_item() {
                handle.parent_handle()
            } else {
                handle.clone()
            }
        };
        let struct_parent_start = self
            .find_structure_ancestor(start)
            .map(get_list_parent_if_is_list_item);
        let struct_parent_next = self
            .find_structure_ancestor(next)
            .map(get_list_parent_if_is_list_item);
        (struct_parent_start, struct_parent_next)
    }

    /// Finds the closest structure node ancestor for the given handle, or None if it doesn't exist
    pub(crate) fn find_structure_ancestor(
        &self,
        handle: &DomHandle,
    ) -> Option<DomHandle> {
        let parent = self.parent(handle);
        if parent.is_structure_node() {
            Some(parent.handle().clone())
        } else if parent.handle().has_parent() {
            self.find_structure_ancestor(&parent.handle())
        } else {
            None
        }
    }

    /// Deletes [from_handle] node appending its children nodes to [to_handle].
    /// Returns a tuple of the index where the children where inserted inside [to_handle] and a
    /// HashMap mapping the old handle of each moved children to its new one.
    fn move_children_and_delete_parent(
        &mut self,
        from_handle: &DomHandle,
        to_handle: &DomHandle,
    ) -> (usize, Vec<(DomHandle, DomHandle)>) {
        let mut moved_handles = Vec::new();
        let ret;
        let children = if let DomNode::Container(from_node) =
            self.lookup_node(from_handle)
        {
            from_node.children().clone()
        } else {
            panic!("Source node must be a ContainerNode");
        };

        if let DomNode::Container(to_node) = self.lookup_node_mut(to_handle) {
            ret = to_node.children().len();
            for c in children {
                let old_handle = c.handle();
                let new_handle = to_node.append_child(c);
                moved_handles.push((old_handle, new_handle));
            }
        } else {
            panic!("Destination node must be a ContainerNode");
        }

        let parent = self.parent_mut(from_handle);
        parent.remove_child(from_handle.index_in_parent());

        (ret, moved_handles)
    }

    fn list_of_ancestors_from_root(handle: &DomHandle) -> Vec<DomHandle> {
        let mut ancestors = Vec::new();
        let mut cur_handle = handle.clone();
        while cur_handle.has_parent() {
            ancestors.push(cur_handle.clone());
            cur_handle = cur_handle.parent_handle();
        }
        // Root node
        ancestors.push(DomHandle::from_raw(Vec::new()));
        // Reverse to start from root
        ancestors.reverse();
        ancestors
    }

    /// Search the supplied iterator for a text node and return a handle to it,
    /// or None if there are no text nodes.
    fn find_text_handle<'a>(
        &self,
        mut locations: impl Iterator<Item = &'a DomLocation>,
    ) -> Option<DomHandle> {
        locations.find_map(|loc| {
            if let DomNode::Text(_) = self.lookup_node(&loc.node_handle) {
                Some(loc.node_handle.clone())
            } else {
                None
            }
        })
    }

    fn first_text_handle(&self, range: &Range) -> Option<DomHandle> {
        self.find_text_handle(range.locations.iter())
    }
}
