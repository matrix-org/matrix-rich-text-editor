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

use std::marker::PhantomData;

use crate::dom::nodes::{ContainerNodeKind, DomNode, TextNode};
use crate::dom::{Dom, DomHandle, DomLocation, MultipleNodesRange, Range};
use crate::UnicodeString;

/// Handles joining together nodes after an edit event.
///
/// For example, if your selection starts in a bold tag, leaves that tag,
/// and then ends up in another bold tag, the final result should be a
/// single bold tag containing all your text.
pub(crate) struct NodeJoiner<S>
where
    S: UnicodeString,
{
    start_handle: DomHandle,

    phantom_data: PhantomData<S>,
}

impl<S> NodeJoiner<S>
where
    S: UnicodeString,
{
    /// Create a NodeJoiner that handles the supplied Range, using the supplied
    /// model to look up nodes where needed.
    pub(crate) fn from_range(dom: &Dom<S>, range: &MultipleNodesRange) -> Self {
        // TODO: for now, to decide whether we will join nodes, we compare
        // the types of the parents of the start and end nodes of the range.
        //
        // There are some ignored tests in test_characters and test_deleting
        // that demonstrate that this is not good enough. We need to consider
        // joining nodes all the way up the tree.

        let start_handle = Self::first_text_handle(dom, range)
            .expect("No start text node found");

        Self {
            start_handle,
            phantom_data: PhantomData::default(),
        }
    }

    /// After the selection range we were given in from_range has been deleted,
    /// join any nodes that match up across the selection.
    pub(crate) fn join_nodes(&self, dom: &mut Dom<S>, new_pos: usize) {
        self.join_structure_nodes(dom, new_pos);
        self.join_format_nodes(dom, new_pos);
    }

    fn join_structure_nodes(&self, dom: &mut Dom<S>, new_pos: usize) {
        // Find next node
        if let Some(mut next_handle) = Self::find_next_node(dom, new_pos) {
            // Find struct parents
            if let (Some(struct_parent_start), Some(struct_parent_next)) =
                Self::find_struct_parents(dom, &self.start_handle, &next_handle)
            {
                if struct_parent_start != struct_parent_next {
                    // Move children
                    let new_index = Self::move_children(
                        dom,
                        &struct_parent_next,
                        &struct_parent_start,
                    );

                    next_handle = struct_parent_start.child_handle(new_index);
                }

                // Find ancestor lists
                let ancestors_start =
                    Self::find_ancestor_list(&self.start_handle);
                let ancestors_next = Self::find_ancestor_list(&next_handle);

                // Merge nodes based on ancestors
                Self::do_join(dom, &ancestors_start, &ancestors_next);
            }
        }
    }

    fn join_format_nodes(&self, dom: &mut Dom<S>, new_pos: usize) {
        if let Some(next_node_handle) = Self::find_next_node(dom, new_pos) {
            self.join_format_nodes_at_level(dom, next_node_handle, 1);
        }
    }

    fn join_format_nodes_at_level(
        &self,
        dom: &mut Dom<S>,
        handle: DomHandle,
        level: usize,
    ) {
        if level >= handle.raw().len() {
            return;
        }
        let cur_handle = DomHandle::from_raw(handle.raw()[..level].to_vec());
        let index_in_parent = cur_handle.index_in_parent();
        if index_in_parent > 0 {
            let prev_handle = cur_handle.prev_sibling();
            if let (
                DomNode::Container(prev_node),
                DomNode::Container(next_node),
            ) = (
                dom.lookup_node(prev_handle.clone()),
                dom.lookup_node(cur_handle.clone()),
            ) {
                if let (
                    ContainerNodeKind::Formatting(prev_format),
                    ContainerNodeKind::Formatting(next_format),
                ) = (prev_node.kind(), next_node.kind())
                {
                    // Found a matching sibling node with the same format
                    if prev_format == next_format {
                        // Move the contents from the current node to the previous one
                        let new_index =
                            Self::move_children(dom, &cur_handle, &prev_handle);
                        // Next iteration
                        let mut cur_path = handle.raw().clone();
                        let prev_path = prev_handle.raw();
                        cur_path[level - 1] = prev_path[level - 1];
                        cur_path[level] = new_index;
                        let new_handle = DomHandle::from_raw(cur_path);

                        self.join_format_nodes_at_level(
                            dom,
                            new_handle,
                            level + 1,
                        );
                    }
                } else {
                    // If both nodes weren't formatting nodes, try at the next level
                    self.join_format_nodes_at_level(dom, handle, level + 1);
                }
            }
        } else {
            // If there's no previous node, try at the next level
            self.join_format_nodes_at_level(dom, handle, level + 1);
        }
    }

    fn do_join(
        dom: &mut Dom<S>,
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

            let start_i = dom.lookup_node(start_handle.clone());
            let next_i = dom.lookup_node(next_handle.clone());

            match (start_i, next_i) {
                (DomNode::Container(start_i), DomNode::Container(next_i)) => {
                    // TODO: check if this is_structure_node verification is needed.
                    if start_i.is_structure_node()
                        && next_i.is_structure_node()
                        && start_i.name() == next_i.name()
                    {
                        // Both containers with the same tag.
                        // Move children from next to start node, remove next node.
                        let new_index_in_parent =
                            Self::move_children(dom, next_handle, start_handle);
                        // This alters ancestors, so we need to re-calculate them and start again.
                        let new_ancestors_next = Self::re_calculate_ancestors(
                            start_handle,
                            new_index_in_parent,
                            i,
                        );
                        // Restart the process
                        Self::do_join(
                            dom,
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
                    let mut new_data = start_i.data().clone();
                    new_data.push_string(&next_i.data());
                    let text_node = DomNode::Text(TextNode::from(new_data));
                    if let DomNode::Container(old_parent) =
                        dom.lookup_node_mut(next_handle.parent_handle())
                    {
                        old_parent.remove_child(next_handle.index_in_parent());
                    }
                    if let DomNode::Container(parent) =
                        dom.lookup_node_mut(start_handle.parent_handle())
                    {
                        parent.replace_child(
                            start_handle.index_in_parent(),
                            vec![text_node],
                        );
                    }
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
        Self::find_ancestor_list(&new_next_handle)
    }

    fn find_next_node(dom: &Dom<S>, pos: usize) -> Option<DomHandle> {
        let new_range = dom.find_range(pos, pos);
        if let Range::SameNode(r) = new_range {
            Some(r.node_handle)
        } else {
            None
        }
    }

    fn find_struct_parents(
        dom: &Dom<S>,
        start: &DomHandle,
        next: &DomHandle,
    ) -> (Option<DomHandle>, Option<DomHandle>) {
        let struct_parent_start = Self::find_struct_parent(dom, start);
        let struct_parent_next = Self::find_struct_parent(dom, next);
        (struct_parent_start, struct_parent_next)
    }

    fn find_struct_parent(
        dom: &Dom<S>,
        handle: &DomHandle,
    ) -> Option<DomHandle> {
        let parent_handle = handle.parent_handle();
        let parent = dom.lookup_node(parent_handle.clone());
        if parent.is_structure_node() && parent_handle.has_parent() {
            if let Some(parent_result) =
                Self::find_struct_parent(dom, &parent_handle)
            {
                Some(parent_result)
            } else {
                Some(parent_handle)
            }
        } else if parent_handle.has_parent() {
            Self::find_struct_parent(dom, &parent_handle)
        } else {
            None
        }
    }

    fn move_children(
        dom: &mut Dom<S>,
        from_handle: &DomHandle,
        to_handle: &DomHandle,
    ) -> usize {
        let ret;
        let children = if let DomNode::Container(from_node) =
            dom.lookup_node(from_handle.clone())
        {
            from_node.children().clone()
        } else {
            panic!("Source node must be a ContainerNode");
        };

        if let DomNode::Container(to_node) =
            dom.lookup_node_mut(to_handle.clone())
        {
            ret = to_node.children().len();
            for c in children {
                to_node.append_child(c)
            }
        } else {
            panic!("Destination node must be a ContainerNode");
        }

        if let DomNode::Container(parent) =
            dom.lookup_node_mut(from_handle.parent_handle())
        {
            parent.remove_child(from_handle.index_in_parent());
        } else {
            panic!("Previous parent of source node must be a ContainerNode");
        }

        ret
    }

    fn find_ancestor_list(handle: &DomHandle) -> Vec<DomHandle> {
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
        dom: &Dom<S>,
        mut locations: impl Iterator<Item = &'a DomLocation>,
    ) -> Option<DomHandle> {
        locations.find_map(|loc| {
            if let DomNode::Text(_) = dom.lookup_node(loc.node_handle.clone()) {
                Some(loc.node_handle.clone())
            } else {
                None
            }
        })
    }

    fn first_text_handle(
        dom: &Dom<S>,
        range: &MultipleNodesRange,
    ) -> Option<DomHandle> {
        Self::find_text_handle(dom, range.locations.iter())
    }
}
