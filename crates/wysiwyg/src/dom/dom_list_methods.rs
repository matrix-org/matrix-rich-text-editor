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

use super::nodes::ContainerNode;
use super::Dom;

impl<S> Dom<S>
where
    S: UnicodeString,
{
    pub fn wrap_nodes_in_list(
        &mut self,
        list_type: ListType,
        handles: Vec<&DomHandle>,
    ) {
        let first_handle = handles[0];
        let mut removed_nodes = Vec::new();
        for handle in handles.iter().rev() {
            removed_nodes.push(self.remove(handle));
        }
        removed_nodes.reverse();

        let mut list_item = ContainerNode::new_list_item(removed_nodes);
        // Set an arbitrary handle allows us to transform this while detached from DOM.
        list_item.set_handle(DomHandle::root());
        let mut list_items = Vec::new();
        let mut line_break_positions = list_item.line_break_positions();

        // Slice the list item on each line break position from last to
        // first position and create a new list item for each slice.
        while let Some(position) = line_break_positions.pop() {
            let mut sliced = list_item.slice_after(position);
            sliced.slice_before(1);
            sliced.add_leading_zwsp();
            list_items.insert(0, DomNode::Container(sliced));
        }

        // Create a list item if we have a non-empty part remaining, or
        // if it's the only part we have (empty list case).
        if list_item.text_len() > 0 || list_items.is_empty() {
            list_item.add_leading_zwsp();
            list_items.insert(0, DomNode::Container(list_item));
        }

        let list = ContainerNode::new_list(list_type, list_items);
        self.insert_at(first_handle, DomNode::Container(list));

        if first_handle.has_parent() {
            self.join_nodes_in_container(&first_handle.parent_handle());
        }
    }

    pub fn remove_list(&mut self, handle: &DomHandle) {
        let list = self.remove(handle);
        let mut nodes_to_insert = Vec::new();
        let DomNode::Container(mut list) = list else {
            panic!("List node is not a container")
        };
        while !list.children().is_empty() {
            let list_item = list.remove_child(0);
            let DomNode::Container(mut list_item) = list_item else {
                panic!("List item is not a container")
            };
            if nodes_to_insert.is_empty() {
                list_item.remove_leading_zwsp();
            } else {
                list_item.replace_leading_zwsp_with_linebreak();
            }

            while !list_item.children().is_empty() {
                nodes_to_insert.push(list_item.remove_child(0));
            }
        }

        self.insert(handle, nodes_to_insert);
        self.join_nodes_in_container(&handle.parent_handle());
    }

    pub fn extract_list_items(
        &mut self,
        handle: &DomHandle,
        start: usize,
        count: usize,
    ) {
        let list = self.lookup_node_mut(handle);
        let DomNode::Container(list) = list else {
            panic!("List is not a container")
        };

        if start == 0 {
            let mut nodes_to_insert = Vec::new();
            for _index in start..start + count {
                let list_item = list.remove_child(start);
                let DomNode::Container(mut list_item) = list_item else {
                    panic!("List item is not a container")
                };
                if nodes_to_insert.is_empty() {
                    list_item.remove_leading_zwsp();
                } else {
                    list_item.replace_leading_zwsp_with_linebreak();
                }
                nodes_to_insert.append(&mut list_item.take_children());
            }
            if list.children().is_empty() {
                // Replace the list if it became empty.
                self.replace(handle, nodes_to_insert);
            } else {
                // Otherwise insert before.
                self.insert(handle, nodes_to_insert);
            }
        } else {
            let mut nodes_to_insert = Vec::new();
            for _index in start..start + count {
                let list_item = list.remove_child(start);
                let DomNode::Container(mut list_item) = list_item else {
                    panic!("List item is not a container")
                };
                if !nodes_to_insert.is_empty() {
                    list_item.replace_leading_zwsp_with_linebreak();
                }
                nodes_to_insert.append(&mut list_item.take_children());
            }

            // Extract further list items to a new list, if any.
            if list.children().len() > start {
                let new_list_children = list.take_children_after(start);
                let new_list = DomNode::new_list(
                    list.get_list_type().expect("Node is not a list").clone(),
                    new_list_children,
                );
                nodes_to_insert.push(new_list);
            }

            self.insert(&handle.next_sibling(), nodes_to_insert);
        }
    }
}

#[cfg(test)]
mod test {
    use widestring::Utf16String;

    use crate::char::CharExt;
    use crate::dom::Dom;
    use crate::tests::testutils_composer_model::{cm, tx};

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
            "<ol><li>~abc<strong>de<em>f</em></strong></li><li><strong><em>~gh</em>i</strong>jkl</li></ol>",
        );

        dom.remove_list(&DomHandle::from_raw(vec![0]));

        assert_eq!(ds(&dom), "abc<strong>de<em>f<br />gh</em>i</strong>jkl",);
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
        assert_eq!(ds(&dom), "<ol><li>~abc</li><li>~def</li></ol>");

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 0, 1);
        assert_eq!(ds(&dom), "abc<ol><li>~def</li></ol>");
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
        assert_eq!(
            ds(&dom),
            "<ol><li>~abc</li><li>~def</li><li>~ghi</li></ol>"
        );

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 0, 2);
        assert_eq!(ds(&dom), "abc<br />def<ol><li>~ghi</li></ol>");
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
        assert_eq!(ds(&dom), "<ol><li>~abc</li><li>~def</li></ol>");

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 1, 1);
        assert_eq!(ds(&dom), "<ol><li>~abc</li></ol>~def");
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
        assert_eq!(
            ds(&dom),
            "<ol><li>~abc</li><li>~def</li><li>~ghi</li></ol>"
        );

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 1, 2);
        assert_eq!(ds(&dom), "<ol><li>~abc</li></ol>~def<br />ghi");
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
            "<ol><li>~abc</li><li>~def</li><li>~ghi</li><li>~jkl</li></ol>"
        );

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 1, 1);
        assert_eq!(
            ds(&dom),
            "<ol><li>~abc</li></ol>~def<ol><li>~ghi</li><li>~jkl</li></ol>"
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
            "<ol><li>~abc</li><li>~def</li><li>~ghi</li><li>~jkl</li></ol>"
        );

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 1, 2);
        assert_eq!(
            ds(&dom),
            "<ol><li>~abc</li></ol>~def<br />ghi<ol><li>~jkl</li></ol>"
        );
    }

    #[test]
    fn extract_single_list_item() {
        let mut dom = cm("abc|").state.dom;
        dom.wrap_nodes_in_list(
            ListType::Ordered,
            vec![&DomHandle::from_raw(vec![0])],
        );
        assert_eq!(ds(&dom), "<ol><li>~abc</li></ol>");

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 0, 1);
        assert_eq!(ds(&dom), "abc");
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
        assert_eq!(
            ds(&dom),
            "<ol><li>~abc</li><li>~def</li><li>~ghi</li></ol>"
        );

        dom.extract_list_items(&DomHandle::from_raw(vec![0]), 0, 3);
        assert_eq!(ds(&dom), "abc<br />def<br />ghi");
    }

    #[test]
    fn wrap_and_remove_lists() {
        list_roundtrips("abc<strong>de<em>f<br />gh</em>i</strong>jkl|");
        list_roundtrips("abc|");
        list_roundtrips("<br />abc<br />|");
        list_roundtrips("<em>ab|c</em>");
        list_roundtrips("<br /><br /><br /><br />|");
        list_roundtrips("<br /><br /><br /><strong><br />|</strong>");
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
        model.state.dom.remove_list(&first_handle);
        assert_eq!(tx(&model), text);
    }

    // TODO: move this to a more globally usable location if needed
    fn ds(dom: &Dom<Utf16String>) -> String {
        dom.to_string()
            .replace(char::zwsp(), "~")
            .replace('\u{A0}', "&nbsp;")
    }
}
