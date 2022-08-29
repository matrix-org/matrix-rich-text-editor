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

use crate::dom::nodes::DomNode;
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
    first_type: Option<String>,
    last_type: Option<String>,

    /// This is just to allow us to have the S generic param
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

        let first_type = Self::first_text_handle(dom, range)
            .map(|h| Self::parent_container_type(dom, h));

        let last_type = Self::last_text_handle(dom, range)
            .map(|h| Self::parent_container_type(dom, h));

        Self {
            first_type,
            last_type,
            phantom_data: PhantomData::default(),
        }
    }

    /// After the selection range we were given in from_range has been deleted,
    /// join any nodes that match up across the selection.
    pub(crate) fn join_nodes(&self, dom: &mut Dom<S>, new_pos: usize) {
        match (self.first_type.as_ref(), self.last_type.as_ref()) {
            (Some(first_type), Some(last_type)) => {
                if first_type == last_type {
                    // The selection started and ended in the same type of
                    // node - we should join these 2 tags.
                    Self::join_nodes_of_same_type(dom, new_pos)
                }
            }
            _ => {}
        }
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
