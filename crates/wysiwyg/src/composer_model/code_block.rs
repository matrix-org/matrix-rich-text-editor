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

use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::dom_node::DomNodeKind::ListItem;
use crate::dom::nodes::{DomNode, TextNode};
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{DomHandle, DomLocation, Range};
use crate::{
    ComposerAction, ComposerModel, ComposerUpdate, Location, UnicodeString,
};
use std::cmp::{max, min};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn code_block(&mut self) -> ComposerUpdate<S> {
        if self.action_is_reversed(ComposerAction::CodeBlock) {
            // TODO: add code block removal
            ComposerUpdate::keep()
        } else {
            self.add_code_block()
        }
    }

    fn add_code_block(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let Some((parent_handle, idx_start, idx_end)) = self.find_nodes_to_wrap(s, e) else {
            return ComposerUpdate::keep();
        };
        let mut leaves_to_add = Vec::new();
        let start_handle = parent_handle.child_handle(idx_start);
        let up_to_handle = parent_handle.child_handle(idx_end + 1);
        let iter = self.state.dom.iter_from_handle(&start_handle);
        for node in iter {
            if node.handle() >= up_to_handle {
                break;
            }
            if matches!(node.kind(), DomNodeKind::Text | DomNodeKind::LineBreak)
            {
                leaves_to_add.push(node.clone());
            }
        }

        for i in (idx_start..=idx_end).rev() {
            self.state.dom.remove(&parent_handle.child_handle(i));
        }

        self.state
            .dom
            .insert_at(&start_handle, DomNode::new_code_block(leaves_to_add));

        self.create_update_replace_all()
    }

    fn find_nodes_to_wrap(
        &self,
        start: usize,
        end: usize,
    ) -> Option<(DomHandle, usize, usize)> {
        let range = self.state.dom.find_range(start, end);
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        if leaves.is_empty() {
            None
        } else {
            let first_leaf = leaves.first().unwrap();
            let last_leaf = leaves.last().unwrap();
            let iter = self.state.dom.iter_from_handle(&last_leaf.node_handle);
            let rev_iter = self
                .state
                .dom
                .iter_from_handle(&first_leaf.node_handle)
                .rev();
            let mut nodes_to_cover = (
                HandleWithKind {
                    handle: first_leaf.node_handle.clone(),
                    kind: first_leaf.kind.clone(),
                },
                HandleWithKind {
                    handle: last_leaf.node_handle.clone(),
                    kind: last_leaf.kind.clone(),
                },
            );
            let selection_contains_several_list_items = range
                .locations
                .iter()
                .filter(|l| l.kind == ListItem)
                .count()
                > 1;
            for node in rev_iter {
                if !node.is_block_node()
                    && node.kind() != DomNodeKind::LineBreak
                    && (node.kind() != ListItem
                        || selection_contains_several_list_items)
                {
                    nodes_to_cover.0 = HandleWithKind {
                        handle: node.handle(),
                        kind: node.kind().clone(),
                    };
                } else {
                    break;
                }
            }

            for node in iter {
                let handle = node.handle();
                if !node.is_block_node()
                    && node.kind() != DomNodeKind::LineBreak
                    && (node.kind() != ListItem
                        || selection_contains_several_list_items)
                {
                    nodes_to_cover.1 = HandleWithKind {
                        handle: node.handle(),
                        kind: node.kind().clone(),
                    };
                } else {
                    break;
                }
            }

            let (first, last) = nodes_to_cover;
            let max_level =
                min(first.handle.raw().len(), last.handle.raw().len());
            let mut min_level = 0;
            for i in min_level..max_level {
                min_level = i;
                if first.handle.raw()[i] != last.handle.raw()[i] {
                    break;
                }
            }
            if first.kind == ListItem || last.kind == ListItem {
                // We should wrap their parent List instead
                min_level -= 1;
            }
            let idx_start = first.handle.raw()[min_level];
            let idx_end = last.handle.raw()[min_level];
            let ancestor_handle = first.handle.sub_handle_up_to(min_level);
            Some((ancestor_handle, idx_start, idx_end))
        }
    }
}

struct HandleWithKind {
    handle: DomHandle,
    kind: DomNodeKind,
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::{cm, tx};
    use crate::DomHandle;

    #[test]
    fn find_ranges_to_wrap_simple_text() {
        let model = cm("Some text|");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(Vec::new()), 0, 0)));
    }

    #[test]
    fn find_ranges_to_wrap_several_nodes() {
        let model = cm("Some text| <b>and bold </b><i>and italic</i>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(Vec::new()), 0, 2)));
    }

    #[test]
    fn find_ranges_to_wrap_several_nodes_with_line_break_at_end() {
        let model = cm("Some text| <b>and bold </b><br/><i>and italic</i>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(Vec::new()), 0, 1)));
    }

    #[test]
    fn find_ranges_to_wrap_several_nodes_with_line_break_at_start() {
        let model = cm("Some text <br/><b>and bold </b><i>|and italic</i>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(Vec::new()), 2, 3)));
    }

    #[test]
    fn find_ranges_to_wrap_list_item() {
        let model = cm(
            "<ul><li>Some text <b>and bold </b><i>|and italic</i></li></ul>",
        );
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(vec![0, 0]), 0, 2)));
    }

    #[test]
    fn find_ranges_to_wrap_list_item_with_line_breaks() {
        let model = cm(
            "<ul><li>Some text <b>and bold </b><br/><i>and| italic</i></li></ul>",
        );
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(vec![0, 0]), 3, 3)));
    }

    #[test]
    fn find_ranges_to_wrap_several_list_items() {
        let model = cm("<ul><li>{First item</li><li>Second}| item</li></ul>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(vec![]), 0, 0)));
    }

    #[test]
    fn find_ranges_to_wrap_list_and_external_nodes() {
        let model =
            cm("{Text <ul><li>First item</li><li>Second}| item</li></ul>");
        let (s, e) = model.safe_selection();
        let ret = model.find_nodes_to_wrap(s, e);
        assert_eq!(ret, Some((DomHandle::from_raw(vec![]), 0, 1)));
    }

    // Tests: Code block

    #[test]
    fn add_code_block_to_simple_text() {
        let mut model = cm("Some text|");
        model.code_block();
        assert_eq!(tx(&model), "<pre>Some text|</pre>");
    }

    #[test]
    fn add_code_block_to_several_nodes() {
        let mut model = cm("Some text| <b>and bold </b><i>and italic</i>");
        model.code_block();
        assert_eq!(tx(&model), "<pre>Some text| and bold and italic</pre>");
    }

    #[test]
    fn add_code_block_to_several_nodes_with_line_break_at_end() {
        let mut model = cm("Some text| <b>and bold </b><br/><i>and italic</i>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "<pre>Some text| and bold&nbsp;</pre><br /><i>and italic</i>"
        );
    }

    #[test]
    fn add_code_block_to_several_nodes_with_line_break_at_start() {
        let mut model = cm("Some text <br/><b>and bold </b><i>and |italic</i>");
        model.code_block();
        assert_eq!(
            tx(&model),
            "Some text <br /><pre>and bold and |italic</pre>"
        );
    }
}
