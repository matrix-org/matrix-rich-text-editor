// Copyright 2023 The Matrix.org Foundation C.I.C.
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

use crate::{DomHandle, DomNode, UnicodeString};

use super::{Dom, Range};

impl<S> Dom<S>
where
    S: UnicodeString,
{
    pub fn insert_node_at_range(
        &mut self,
        range: &Range,
        new_node: DomNode<S>,
    ) -> DomHandle {
        if range.is_cursor() {
            self.insert_node_at_cursor(range, new_node)
        } else {
            self.insert_node_at_selection(range, new_node)
        }
    }

    fn insert_node_at_cursor(
        &mut self,
        range: &Range,
        new_node: DomNode<S>,
    ) -> DomHandle {
        // manipulate the state of the dom as required
        if let Some(leaf) = range.leaves().next() {
            // when we have a leaf, the way we treat the insertion depends on the cursor position inside that leaf
            let cursor_at_end = leaf.start_offset == leaf.length;
            let cursor_at_start = leaf.start_offset == 0;

            if cursor_at_start {
                // insert the new node before a leaf that contains a cursor at the start
                self.insert_at(&leaf.node_handle, new_node)
            } else if cursor_at_end {
                // insert the new node after a leaf that contains a cursor at the end
                self.append(&self.parent(&leaf.node_handle).handle(), new_node)
            } else {
                // otherwise insert the new node in the middle of a text node
                self.insert_into_text(
                    &leaf.node_handle,
                    leaf.start_offset,
                    new_node,
                )
            }
        } else {
            // if we haven't found a leaf node, try to find a container node
            let first_location = range.locations.first();

            match first_location {
                // if we haven't found anything, we're inserting into an empty dom
                None => self.append_at_end_of_document(new_node),
                Some(container) => {
                    self.append(&container.node_handle, new_node)
                }
            }
        }
    }

    fn insert_node_at_selection(
        &mut self,
        range: &Range,
        mut new_node: DomNode<S>,
    ) -> DomHandle {
        // TODO
        return DomHandle::new_unset();
    }
}
