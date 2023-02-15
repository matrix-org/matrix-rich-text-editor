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

use core::panic;

use crate::{DomHandle, DomNode, ListType, UnicodeString};

use super::nodes::dom_node::DomNodeKind::{CodeBlock, Quote};
use super::nodes::ContainerNode;
use super::Dom;

impl<S> Dom<S>
where
    S: UnicodeString,
{
    /// Wrap nodes at given handles into a new list.
    ///
    /// * `list_type` - the type of list to create (ordered/unordered).
    /// * `handles` - vec containing all the node handles.
    pub fn wrap_nodes_in_list(
        &mut self,
        list_type: ListType,
        handles: Vec<&DomHandle>,
    ) {
        if handles.is_empty() {
            let empty_list_item = DomNode::new_list_item(Vec::new());
            let list = DomNode::new_list(list_type, vec![empty_list_item]);
            self.append_at_end_of_document(list);
            return;
        }

        let first_handle = handles[0];
        let mut removed_nodes = Vec::new();
        for handle in handles.iter().rev() {
            let removed = self.remove(handle);
            // Quotes and code block contains paragraphs as direct children
            // We need to wrap these instead of the quote/code block
            //
            // Note: this behaviour might change if we want to handle
            // quote/code blocks inside list items.
            if removed.kind() == Quote || removed.kind() == CodeBlock {
                if let DomNode::Container(c) = removed {
                    let mut children = c.take_children();
                    children.reverse();
                    removed_nodes.append(&mut children);
                } else {
                    panic!("Quote/code block is not a container!")
                }
            } else {
                removed_nodes.push(removed);
            }
        }
        removed_nodes.reverse();

        let mut list_items = Vec::new();
        if removed_nodes.iter().all(|n| n.is_block_node()) {
            for block_node in removed_nodes {
                let DomNode::Container(block_node) = block_node else {
                    panic!("Block node must be a container node")
                };
                let children = block_node.take_children();
                let list_item = DomNode::new_list_item(children);
                list_items.push(list_item);
            }
        } else {
            let mut list_item = ContainerNode::new_list_item(removed_nodes);
            // Set an arbitrary handle allows us to transform this while detached from DOM.
            list_item.set_handle(DomHandle::root());
            let mut line_break_positions = list_item.line_break_positions();

            // Slice the list item on each line break position from last to
            // first position and create a new list item for each slice.
            while let Some(position) = line_break_positions.pop() {
                let mut sliced = list_item.slice_after(position);
                sliced.slice_before(1);
                if sliced.children().len() == 1
                    && sliced.children()[0].is_line_break()
                {
                    list_items.insert(0, DomNode::new_list_item(Vec::new()));
                } else {
                    list_items.insert(0, DomNode::Container(sliced));
                }
            }

            // Create a list item if we have a non-empty part remaining, or
            // if it's the only part we have (empty list case).
            if list_item.text_len() > 0 || list_items.is_empty() {
                list_items.insert(0, DomNode::Container(list_item));
            }
        }

        let list = ContainerNode::new_list(list_type, list_items);
        self.insert_at(first_handle, DomNode::Container(list));

        if first_handle.has_parent() {
            self.join_nodes_in_container(&first_handle.parent_handle());
        }
    }

    /// Extract all items from the list at given handle and move
    /// them appropriately into the DOM. Extracted list items are
    /// separated by line breaks.
    ///
    /// * `handle` - the list handle.
    pub fn extract_from_list(&mut self, handle: &DomHandle) {
        let list = self.lookup_node(handle);
        let DomNode::Container(list) = list else {
            panic!("List is not a container")
        };
        self.extract_list_items(handle, 0, list.children().len());
    }

    /// Extract items from the list at given handle and positions
    /// and move them appropriately into the DOM. Extracted list
    /// items are separated by line breaks.
    ///
    /// * `handle` - the list handle.
    /// * `child_index` - child index at which the extraction should start.
    /// * `count` - number of children that should be extracted.
    pub fn extract_list_items(
        &mut self,
        handle: &DomHandle,
        child_index: usize,
        count: usize,
    ) {
        fn wrap_children<S: UnicodeString>(
            children: Vec<DomNode<S>>,
        ) -> Vec<DomNode<S>> {
            if children.is_empty()
                || children.iter().any(|n| !n.is_block_node())
            {
                vec![DomNode::new_paragraph(children)]
            } else {
                children
            }
        }
        let list = self.lookup_node_mut(handle);
        let DomNode::Container(list) = list else {
            panic!("List is not a container")
        };

        let mut nodes_to_insert = Vec::new();
        for _index in child_index..child_index + count {
            let list_item = list.remove_child(child_index);
            let DomNode::Container(list_item) = list_item else {
                panic!("List item is not a container")
            };
            let children = list_item.take_children();
            nodes_to_insert.append(&mut wrap_children(children));
        }
        if child_index == 0 {
            if list.children().is_empty() {
                // Replace the list if it became empty.
                self.replace(handle, nodes_to_insert);
            } else {
                // Otherwise insert before.
                self.insert(handle, nodes_to_insert);
            }
        } else {
            // Extract further list items to a new list, if any.
            if list.children().len() > child_index {
                let new_list_children = list.take_children_after(child_index);
                let new_list = DomNode::new_list(
                    list.get_list_type().expect("Node is not a list").clone(),
                    new_list_children,
                );
                nodes_to_insert.push(new_list);
            }

            self.insert(&handle.next_sibling(), nodes_to_insert);
        }
        self.join_nodes_in_container(&handle.parent_handle());
    }

    /// Slice list item at given handle and offset.
    /// * `handle` - the list item handle.
    /// * `offset` - offset at which the list item should be sliced
    #[allow(dead_code)]
    pub(crate) fn slice_list_item(
        &mut self,
        handle: &DomHandle,
        offset: usize,
    ) {
        let list_item = self.lookup_node_mut(handle);
        let slice = list_item.slice_after(offset);
        let list = self.lookup_node_mut(&handle.parent_handle());
        let DomNode::Container(list) = list else { panic!("List node is not a container") };
        list.insert_child(handle.index_in_parent() + 1, slice);
        self.join_nodes_in_container(&handle.parent_handle());
    }
}

#[cfg(test)]
mod test {
    use widestring::Utf16String;

    use crate::dom::Dom;
    use crate::tests::testutils_composer_model::{cm, tx};

    use crate::char::CharExt;
    use crate::{DomHandle, ListType};

    #[test]
    fn wrap_consecutive_nodes_in_list() {
        let mut dom = cm("abc<strong>de<em>f<br />gh</em>i</strong>jkl|")
            .state
            .dom;
        dom.wrap_nodes_in_list(
            ListType::Ordered,
            vec![
                &DomHandle::from_raw(vec![0]),
                &DomHandle::from_raw(vec![1]),
                &DomHandle::from_raw(vec![2]),
            ],
        );
        assert_eq!(
            ds(&dom),
            "<ol><li>abc<strong>de<em>f</em></strong></li><li><strong><em>gh</em>i</strong>jkl</li></ol>",
        );

        dom.extract_from_list(&DomHandle::from_raw(vec![0]));

        assert_eq!(ds(&dom), "<p>abc<strong>de<em>f</em></strong></p><p><strong><em>gh</em>i</strong>jkl</p>",);
    }

    #[test]
    fn extract_first_list_item() {
        let mut dom = cm("abc<br />def|").state.dom;
        dom.wrap_nodes_in_list(
            ListType::Ordered,
            vec![
                &DomHandle::from_raw(vec![0]),
                &DomHandle::from_raw(vec![1]),
                &DomHandle::from_raw(vec![2]),
            ],
        );
        assert_eq!(ds(&dom), "<ol><li>abc</li><li>def</li></ol>");

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 0, 1);
        assert_eq!(ds(&dom), "<p>abc</p><ol><li>def</li></ol>");
    }

    #[test]
    fn extract_first_list_items() {
        let mut dom = cm("abc<br />def<br />ghi|").state.dom;
        dom.wrap_nodes_in_list(
            ListType::Ordered,
            vec![
                &DomHandle::from_raw(vec![0]),
                &DomHandle::from_raw(vec![1]),
                &DomHandle::from_raw(vec![2]),
                &DomHandle::from_raw(vec![3]),
                &DomHandle::from_raw(vec![4]),
            ],
        );
        assert_eq!(ds(&dom), "<ol><li>abc</li><li>def</li><li>ghi</li></ol>");

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 0, 2);
        assert_eq!(ds(&dom), "<p>abc</p><p>def</p><ol><li>ghi</li></ol>");
    }

    #[test]
    fn extract_last_list_item() {
        let mut dom = cm("abc<br />def|").state.dom;
        dom.wrap_nodes_in_list(
            ListType::Ordered,
            vec![
                &DomHandle::from_raw(vec![0]),
                &DomHandle::from_raw(vec![1]),
                &DomHandle::from_raw(vec![2]),
            ],
        );
        assert_eq!(ds(&dom), "<ol><li>abc</li><li>def</li></ol>");

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 1, 1);
        assert_eq!(ds(&dom), "<ol><li>abc</li></ol><p>def</p>");
    }

    #[test]
    fn extract_last_list_items() {
        let mut dom = cm("abc<br />def<br />ghi|").state.dom;
        dom.wrap_nodes_in_list(
            ListType::Ordered,
            vec![
                &DomHandle::from_raw(vec![0]),
                &DomHandle::from_raw(vec![1]),
                &DomHandle::from_raw(vec![2]),
                &DomHandle::from_raw(vec![3]),
                &DomHandle::from_raw(vec![4]),
            ],
        );
        assert_eq!(ds(&dom), "<ol><li>abc</li><li>def</li><li>ghi</li></ol>");

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 1, 2);
        assert_eq!(ds(&dom), "<ol><li>abc</li></ol><p>def</p><p>ghi</p>");
    }

    #[test]
    fn extract_middle_list_item() {
        let mut dom = cm("abc<br />def<br />ghi<br />jkl|").state.dom;
        dom.wrap_nodes_in_list(
            ListType::Ordered,
            vec![
                &DomHandle::from_raw(vec![0]),
                &DomHandle::from_raw(vec![1]),
                &DomHandle::from_raw(vec![2]),
                &DomHandle::from_raw(vec![3]),
                &DomHandle::from_raw(vec![4]),
                &DomHandle::from_raw(vec![5]),
                &DomHandle::from_raw(vec![6]),
            ],
        );
        assert_eq!(
            ds(&dom),
            "<ol><li>abc</li><li>def</li><li>ghi</li><li>jkl</li></ol>"
        );

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 1, 1);
        assert_eq!(
            ds(&dom),
            "<ol><li>abc</li></ol><p>def</p><ol><li>ghi</li><li>jkl</li></ol>"
        );
    }

    #[test]
    fn extract_middle_list_items() {
        let mut dom = cm("abc<br />def<br />ghi<br />jkl|").state.dom;
        dom.wrap_nodes_in_list(
            ListType::Ordered,
            vec![
                &DomHandle::from_raw(vec![0]),
                &DomHandle::from_raw(vec![1]),
                &DomHandle::from_raw(vec![2]),
                &DomHandle::from_raw(vec![3]),
                &DomHandle::from_raw(vec![4]),
                &DomHandle::from_raw(vec![5]),
                &DomHandle::from_raw(vec![6]),
            ],
        );
        assert_eq!(
            ds(&dom),
            "<ol><li>abc</li><li>def</li><li>ghi</li><li>jkl</li></ol>"
        );

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 1, 2);
        assert_eq!(
            ds(&dom),
            "<ol><li>abc</li></ol><p>def</p><p>ghi</p><ol><li>jkl</li></ol>"
        );
    }

    #[test]
    fn extract_single_list_item() {
        let mut dom = cm("abc|").state.dom;
        dom.wrap_nodes_in_list(
            ListType::Ordered,
            vec![&DomHandle::from_raw(vec![0])],
        );
        assert_eq!(ds(&dom), "<ol><li>abc</li></ol>");

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 0, 1);
        assert_eq!(ds(&dom), "<p>abc</p>");
    }

    #[test]
    fn extract_entire_list() {
        let mut dom = cm("abc<br />def<br />ghi|").state.dom;
        dom.wrap_nodes_in_list(
            ListType::Ordered,
            vec![
                &DomHandle::from_raw(vec![0]),
                &DomHandle::from_raw(vec![1]),
                &DomHandle::from_raw(vec![2]),
                &DomHandle::from_raw(vec![3]),
                &DomHandle::from_raw(vec![4]),
            ],
        );
        assert_eq!(ds(&dom), "<ol><li>abc</li><li>def</li><li>ghi</li></ol>");

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 0, 3);
        assert_eq!(ds(&dom), "<p>abc</p><p>def</p><p>ghi</p>");
    }

    #[test]
    fn slice_list_item() {
        let mut dom = cm("<em>abcd</em>ef|").state.dom;
        dom.wrap_nodes_in_list(
            ListType::Ordered,
            vec![&DomHandle::from_raw(vec![0]), &DomHandle::from_raw(vec![1])],
        );
        assert_eq!(ds(&dom), "<ol><li><em>abcd</em>ef</li></ol>");

        dom.slice_list_item(&DomHandle::from_raw(vec![0, 0]), 3);
        assert_eq!(
            ds(&dom),
            "<ol><li><em>abc</em></li><li><em>d</em>ef</li></ol>"
        );
    }

    #[test]
    fn wrap_and_remove_lists() {
        // Note: creating a list might consume e.g. a line break and not restore it
        // It's probably not worth it to fix these roundtrips as this wouldn't happen with paragraphs.
        list_roundtrips(
            "<p>abc<strong>de<em>f</em></strong></p><p><strong><em>gh</em>i</strong>jkl|</p>",
        );
        list_roundtrips("<p>abc|</p>");
        list_roundtrips("<p>&nbsp;</p><p>abc</p><p>&nbsp;|</p>");
        list_roundtrips("<p><em>ab|c</em></p>");
        list_roundtrips("<p>{&nbsp;</p><p>&nbsp;}|</p>");
    }

    fn list_roundtrips(text: &str) {
        let mut model = cm(text);
        let (s, e) = model.safe_selection();
        let range = model.state.dom.find_extended_range(s, e);
        let handles: Vec<&DomHandle> = range
            .top_level_locations()
            .map(|l| &l.node_handle)
            .collect();
        let first_handle = handles[0].clone();
        model
            .state
            .dom
            .wrap_nodes_in_list(ListType::Ordered, handles);
        model.state.dom.extract_from_list(&first_handle);
        assert_eq!(tx(&model), text);
    }

    // TODO: move this to a more globally usable location if needed
    fn ds(dom: &Dom<Utf16String>) -> String {
        dom.to_string().replace(char::nbsp(), "&nbsp;")
    }
}
