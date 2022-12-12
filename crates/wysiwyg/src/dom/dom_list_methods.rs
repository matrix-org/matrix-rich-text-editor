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

use crate::{DomHandle, DomNode, ListType, ToHtml, UnicodeString};

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
        offset_before: usize,
        _offset_after: usize,
    ) -> (usize, usize) {
        let offset_before_correction = if offset_before == 0 { 0 } else { 1 };
        // Always equal to 1 ?
        let offset_after_correction = 1;
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

        while let Some(position) = line_break_positions.pop() {
            println!("{}", list_item.to_html());
            let prev_length = list_item.text_len();
            let mut sliced = list_item.slice_after(position);
            assert!(list_item.text_len() == position);
            println!("{}", list_item.to_html());
            assert!(sliced.text_len() == prev_length - position);
            sliced.slice_before(1);
            sliced.add_leading_zwsp();
            assert!(sliced.text_len() == prev_length - position);
            list_items.insert(0, DomNode::Container(sliced));
        }

        if list_item.text_len() > 0 {
            list_item.add_leading_zwsp();
            list_items.insert(0, DomNode::Container(list_item));
        }

        let list = ContainerNode::new_list(list_type, list_items);
        self.insert_at(first_handle, DomNode::Container(list));

        (offset_before_correction, offset_after_correction)
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
            0,
            0,
        );
        assert_eq!(
             tx(&model),
             "<ol><li>~abc<strong>de<em>f</em></strong></li><li><strong><em>~gh</em>i</strong>jk|l</li></ol>",
        )
    }
}
