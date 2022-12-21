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
use crate::dom::nodes::dom_node::DomNodeKind::*;
use crate::dom::{Dom, DomLocation, Range};
use crate::{DomHandle, DomNode, UnicodeString};
use std::cmp::min;

impl<S> Dom<S>
where
    S: UnicodeString,
{
    pub(crate) fn find_nodes_to_wrap_in_block(
        &self,
        start: usize,
        end: usize,
    ) -> Option<WrapSearchResult> {
        let range = self.find_range(start, end);
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        if leaves.is_empty() {
            None
        } else {
            let first_leaf = leaves.first().unwrap();
            let last_leaf = leaves.last().unwrap();
            let rev_iter = self.iter_from_handle(&first_leaf.node_handle).rev();
            let iter = self.iter_from_handle(&last_leaf.node_handle);
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
            // Walk backwards from the first leaf until we find the prev line break or block node
            for node in rev_iter {
                if self.should_include_in_nodes_to_wrap(&node, &range) {
                    if node.is_leaf() {
                        nodes_to_cover.0 = HandleWithKind {
                            handle: node.handle(),
                            kind: node.kind().clone(),
                        };
                    }
                } else {
                    break;
                }
            }

            // Walk forwards from the first leaf until we find the next line break or block node
            for node in iter {
                if self.should_include_in_nodes_to_wrap(&node, &range) {
                    if node.is_leaf() {
                        nodes_to_cover.1 = HandleWithKind {
                            handle: node.handle(),
                            kind: node.kind().clone(),
                        };
                    }
                } else {
                    break;
                }
            }

            // Find closest block ancestors for the first and last nodes to cover, then get the
            // shared part of the DomHandle (the closest common ancestor).
            let (first, last) = nodes_to_cover;
            let first_ancestor_to_split =
                self.find_block_ancestor_to_split(&first.handle);
            let last_ancestor_to_split =
                self.find_block_ancestor_to_split(&first.handle);
            let max_depth = min(
                first_ancestor_to_split.depth(),
                last_ancestor_to_split.depth(),
            );
            let mut min_depth = 0;
            for i in min_depth..max_depth {
                min_depth = i;
                if first.handle.raw()[i] != last.handle.raw()[i] {
                    break;
                }
            }

            // Workaround for list items, if the closest list item ancestors for the first and last
            // nodes are the same one, we want to select the contents of the list item instead.
            let first_list_item =
                self.find_parent_list_item_or_self(&first.handle);
            let last_list_item =
                self.find_parent_list_item_or_self(&last.handle);
            if first_list_item.is_some()
                && last_list_item.is_some()
                && first_list_item == last_list_item
            {
                // We should wrap the list item instead
                min_depth += 1;
            }

            // Will wrap an empty text node at the end of the editor
            if first.handle == last.handle && first.kind == LineBreak {
                return None;
            }
            let ancestor_handle = first.handle.sub_handle_up_to(min_depth);
            Some(WrapSearchResult {
                ancestor_handle,
                start_handle: first.handle,
                end_handle: last.handle,
                range,
            })
        }
    }

    fn should_include_in_nodes_to_wrap(
        &self,
        node: &DomNode<S>,
        range: &Range,
    ) -> bool {
        // We don't want to include block nodes
        !node.is_block_node()
            // We should stop at line breaks
            && node.kind() != LineBreak
            // We should stop at list items as long as they're not part of the selection
            && (node.kind() != ListItem || range.contains(&node.handle()))
    }

    pub(crate) fn find_block_ancestor_to_split(
        &self,
        handle: &DomHandle,
    ) -> DomHandle {
        if handle.depth() <= 1 {
            DomHandle::root()
        } else {
            for i in (0..handle.depth()).rev() {
                let ancestor_handle = handle.sub_handle_up_to(i);
                let ancestor = self.lookup_node(&ancestor_handle);
                if ancestor.is_block_node() || ancestor.kind() == ListItem {
                    return ancestor_handle;
                }
            }
            panic!("Should never reach this point, one of the parents surely can be split.");
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tests::testutils_composer_model::cm;
    use crate::DomHandle;

    #[test]
    fn find_ranges_to_wrap_simple_text() {
        let model = cm("Some text|");
        let (s, e) = model.safe_selection();
        let ret = model.state.dom.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(Vec::new()));
        assert_eq!(ret.start_handle, DomHandle::from_raw(vec![0]));
        assert_eq!(ret.start_handle, DomHandle::from_raw(vec![0]));
    }

    #[test]
    fn find_ranges_to_wrap_several_nodes() {
        let model = cm("Some text| <b>and bold </b><i>and italic</i>");
        let (s, e) = model.safe_selection();
        let ret = model.state.dom.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(Vec::new()));
        // assert_eq!(ret.idx_start, 0);
        // assert_eq!(ret.idx_end, 2);
    }

    #[test]
    fn find_ranges_to_wrap_several_nodes_with_line_break_at_end() {
        let model = cm("Some text| <b>and bold </b><br/><i>and italic</i>");
        let (s, e) = model.safe_selection();
        let ret = model.state.dom.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(Vec::new()));
        // assert_eq!(ret.idx_start, 0);
        // assert_eq!(ret.idx_end, 1);
    }

    #[test]
    fn find_ranges_to_wrap_several_nodes_with_line_break_at_start() {
        let model = cm("Some text <br/><b>and bold </b><i>|and italic</i>");
        let (s, e) = model.safe_selection();
        let ret = model.state.dom.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(Vec::new()));
        // assert_eq!(ret.idx_start, 2);
        // assert_eq!(ret.idx_end, 3);
    }

    #[test]
    fn find_ranges_to_wrap_list_item() {
        let model = cm(
            "<ul><li>Some text <b>and bold </b><i>|and italic</i></li></ul>",
        );
        let (s, e) = model.safe_selection();
        let ret = model.state.dom.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(vec![0, 0]));
        // assert_eq!(ret.idx_start, 0);
        // assert_eq!(ret.idx_end, 2);
    }

    #[test]
    fn find_ranges_to_wrap_list_item_with_line_breaks() {
        let model = cm(
            "<ul><li>Some text <b>and bold </b><br/><i>and| italic</i></li></ul>",
        );
        let (s, e) = model.safe_selection();
        let ret = model.state.dom.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(vec![0, 0]));
        // assert_eq!(ret.idx_start, 3);
        // assert_eq!(ret.idx_end, 3);
    }

    #[test]
    fn find_ranges_to_wrap_several_list_items() {
        let model = cm("<ul><li>{First item</li><li>Second}| item</li></ul>");
        let (s, e) = model.safe_selection();
        let ret = model.state.dom.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(vec![0]));
        // assert_eq!(ret.idx_start, 0);
        // assert_eq!(ret.idx_end, 0);
    }

    #[test]
    fn find_ranges_to_wrap_list_and_external_nodes() {
        let model =
            cm("{Text <ul><li>First item</li><li>Second}| item</li></ul>");
        let (s, e) = model.safe_selection();
        let ret = model.state.dom.find_nodes_to_wrap_in_block(s, e).unwrap();
        assert_eq!(ret.ancestor_handle, DomHandle::from_raw(Vec::new()));
        // assert_eq!(ret.idx_start, 0);
        // assert_eq!(ret.idx_end, 1);
    }
}

pub(crate) struct WrapSearchResult {
    pub(crate) ancestor_handle: DomHandle,
    pub(crate) start_handle: DomHandle,
    pub(crate) end_handle: DomHandle,
    pub(crate) range: Range,
}

struct HandleWithKind {
    handle: DomHandle,
    kind: DomNodeKind,
}
