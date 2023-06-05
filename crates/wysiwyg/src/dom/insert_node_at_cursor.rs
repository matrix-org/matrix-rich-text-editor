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
    // Inserts the new node at the current cursor position if possible, panics if
    // the range passed is a selection
    pub fn insert_node_at_cursor(
        &mut self,
        range: &Range,
        new_node: DomNode<S>,
    ) -> DomHandle {
        if range.is_selection() {
            panic!("Attempted to use `insert_node_at_cursor` with a selection")
        }

        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();

        let inserted_handle: DomHandle;

        // manipulate the state of the dom as required
        if let Some(leaf) = range.leaves().next() {
            // when we have a leaf, the way we treat the insertion depends on the cursor position inside that leaf
            let cursor_at_end = leaf.start_offset == leaf.length;
            let cursor_at_start = leaf.start_offset == 0;

            let leaf_is_placeholder =
                self.lookup_node(&leaf.node_handle).is_placeholder();

            // special case where we replace a paragraph placeholder
            if leaf_is_placeholder || cursor_at_start {
                // insert the new node before a leaf that contains a cursor at the start
                inserted_handle = self.insert_at(&leaf.node_handle, new_node);
            } else if cursor_at_end {
                // insert the new node after a leaf that contains a cursor at the end
                inserted_handle = self
                    .append(&self.parent(&leaf.node_handle).handle(), new_node);
            } else {
                // otherwise insert the new node in the middle of a text node
                inserted_handle = self.insert_into_text(
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
                None => {
                    inserted_handle = self.append_at_end_of_document(new_node);
                }
                Some(container) => {
                    inserted_handle =
                        self.append(&container.node_handle, new_node);
                }
            };
        }

        #[cfg(any(test, feature = "assert-invariants"))]
        self.assert_invariants();

        inserted_handle
    }
}

#[cfg(test)]
mod test {
    use crate::{
        tests::{testutils_composer_model::cm, testutils_conversion::utf16},
        DomNode, ToHtml,
    };
    #[test]
    #[should_panic]
    fn panics_if_passed_selection() {
        let mut model = cm("{something}|");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );
    }

    #[test]
    fn inserts_node_in_empty_model() {
        let mut model = cm("|");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );

        assert_eq!(model.state.dom.to_html(), "<a href=\"href\"></a>")
    }

    #[test]
    fn inserts_node_into_empty_container() {
        let mut model = cm("<p>|</p>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );

        assert_eq!(model.state.dom.to_html(), "<p><a href=\"href\"></a></p>")
    }

    #[test]
    fn inserts_node_into_leaf_start() {
        let mut model = cm("<p>|this is a leaf</p>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            "<p><a href=\"href\"></a>this is a leaf</p>"
        )
    }

    #[test]
    fn inserts_node_into_leaf_middle() {
        let mut model = cm("<p>this is| a leaf</p>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            "<p>this is<a href=\"href\"></a> a leaf</p>"
        )
    }

    #[test]
    fn inserts_node_into_leaf_end() {
        let mut model = cm("<p>this is a leaf|</p>");
        let (start, end) = model.safe_selection();
        let range = model.state.dom.find_range(start, end);

        model.state.dom.insert_node_at_cursor(
            &range,
            DomNode::new_link(utf16("href"), vec![], vec![]),
        );

        assert_eq!(
            model.state.dom.to_html(),
            "<p>this is a leaf<a href=\"href\"></a></p>"
        )
    }
}
