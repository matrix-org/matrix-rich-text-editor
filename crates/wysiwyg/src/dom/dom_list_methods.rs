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

        if list_item.text_len() > 0 {
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
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::{cm, tx};

    use crate::{DomHandle, ListType};

    #[test]
    fn wrap_consecutive_nodes_in_list() {
        let mut model = cm("abc<strong>de<em>f<br />gh</em>i</strong>jkl|");
        model.state.dom.wrap_nodes_in_list(
            ListType::Ordered,
            vec![
                &DomHandle::from_raw(vec![0]),
                &DomHandle::from_raw(vec![1]),
                &DomHandle::from_raw(vec![2]),
            ],
        );
        assert_eq!(
             tx(&model),
             "<ol><li>~abc<strong>de<em>f</em></strong></li><li><strong><em>~gh</em>i</strong>jk|l</li></ol>",
        );

        model.state.dom.remove_list(&DomHandle::from_raw(vec![0]));

        assert_eq!(tx(&model), "abc<strong>de<em>f<br />gh</em>i</strong>jkl|",);
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
}
