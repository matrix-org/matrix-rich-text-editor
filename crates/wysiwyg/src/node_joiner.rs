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

use crate::dom::nodes::{DomNode, TextNode};
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
                    if start_i.name() == next_i.name() {
                        // Both containers with the same tag.
                        // Move children from next to start node, remove next node.
                        let new_index_in_parent =
                            Self::move_children(dom, next_handle, start_handle);
                        // This alters ancestors, so we need to re-calculate them and start again.
                        // TODO: maybe move this to another fn?
                        let mut new_next_path =
                            start_handle.raw()[..i].to_vec();
                        new_next_path.push(new_index_in_parent);
                        let new_next_handle =
                            DomHandle::from_raw(new_next_path);

                        // Re-calculate ancestors
                        let new_ancestors_next =
                            Self::find_ancestor_list(&new_next_handle);
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
                (DomNode::Text(start_i), DomNode::Text(next_i)) => {
                    let mut new_data = start_i.data().clone();
                    new_data.push_string(&next_i.data());
                    let text_node = DomNode::Text(TextNode::from(new_data));
                    if let DomNode::Container(parent) =
                        dom.lookup_node_mut(start_handle.parent_handle())
                    {
                        parent.remove_child(next_handle.index_in_parent());
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

    fn join_nodes_of_same_type(dom: &mut Dom<S>, new_pos: usize) {
        let new_range = dom.find_range(new_pos, new_pos);

        if let Range::SameNode(new_range) = new_range {
            // Find the text node we are in now the selection has been deleted
            let current_text = new_range.node_handle;

            // Find the container node that contains us e.g. <b>. This is
            // what we are going to join with something else>
            let current_container = current_text.parent_handle();

            // Find the parent that contains both the <b> tags that we are
            // going to join.
            let parent = current_container.parent_handle();

            // Look for our sibling container - e.g. the other <b>
            // TODO: check we have a sibling instead of crashing here
            let sibling =
                parent.child_handle(current_container.index_in_parent() + 1);

            Self::join_nodes_using_handles(dom, parent, current_text, sibling);
        } else {
            // We won't get here until we delete SameNode, or allow pasting
            // HTML formatted text
            panic!(
                "Not set up for MultipleNodeRange, but we could use
                   first_text_handle here to get the right node I think."
            );
        }
    }

    fn join_nodes_using_handles(
        dom: &mut Dom<S>,
        parent: DomHandle,
        current_text: DomHandle,
        sibling: DomHandle,
    ) {
        let sibling_node = dom.lookup_node(sibling.clone());
        if let DomNode::Container(_) = sibling_node {
            let sibling_text = sibling.child_handle(0);
            let sibling_text_node = dom.lookup_node(sibling_text);
            if let DomNode::Text(second_text_node) = sibling_text_node {
                let remaining_text = second_text_node.data().clone();
                Self::do_join_nodes(
                    dom,
                    parent,
                    current_text,
                    sibling,
                    remaining_text,
                );
            } else {
                // TODO: The first node in the sibling container was not text.
                // We should probably just combine the children, but bailing
                // out for now.
            }
        } else {
            // TODO: Sibling is a text node - should probably join to
            // it, but bailing out for now.
        }
    }

    fn is_container(dom: &Dom<S>, handle: &DomHandle) -> bool {
        matches!(dom.lookup_node(handle.clone()), DomNode::Container(_))
    }

    /// Actually join the nodes: delete the sibling, and append its text onto
    /// our current text node.
    fn do_join_nodes(
        dom: &mut Dom<S>,
        parent: DomHandle,
        current_text: DomHandle,
        sibling: DomHandle,
        remaining_text: S,
    ) {
        // Delete the sibling's container
        let parent_node = dom.lookup_node_mut(parent);
        if let DomNode::Container(parent_node) = parent_node {
            parent_node.remove_child(sibling.index_in_parent())
        } else {
            panic!("Parent was not a container!");
        }

        // Add the remaining text to this node
        let current_text = dom.lookup_node_mut(current_text);
        if let DomNode::Text(current_text) = current_text {
            let mut new_data = current_text.data().clone();
            new_data.push_string(&remaining_text);
            current_text.set_data(new_data);
        } else {
            panic!("Current text was not text!");
        }
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

    fn last_text_handle(
        dom: &Dom<S>,
        range: &MultipleNodesRange,
    ) -> Option<DomHandle> {
        Self::find_text_handle(dom, range.locations.iter().rev())
    }

    /// Panics if this handle does not refer to something with a parent
    fn parent_container_type(dom: &Dom<S>, handle: DomHandle) -> String {
        let parent = handle.parent_handle();
        let node = dom.lookup_node(parent);
        if let DomNode::Container(node) = node {
            node.name().to_utf8()
        } else {
            panic!("Expected parent to be a container node");
        }
    }
}
