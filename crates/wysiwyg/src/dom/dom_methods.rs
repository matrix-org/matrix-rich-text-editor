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

use crate::{DomHandle, DomNode, UnicodeString};

use super::nodes::ContainerNode;
use super::unicode_string::UnicodeStringExt;
use super::Dom;

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
}

/// If the child of parent at index and the one after it are both text
/// nodes, merge them into the first and delete the second.
fn merge_if_adjacent_text_nodes<S>(parent: &mut ContainerNode<S>, index: usize)
where
    S: UnicodeString,
{
    let last_inserted_child = parent.children().get(index);
    let after_child = parent.children().get(index + 1);
    if let (Some(DomNode::Text(t1)), Some(DomNode::Text(t2))) =
        (last_inserted_child, after_child)
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
