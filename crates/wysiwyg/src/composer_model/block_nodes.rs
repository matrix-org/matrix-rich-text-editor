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
use crate::dom::{DomLocation, Range};
use crate::{ComposerModel, DomHandle, UnicodeString};
use std::cmp::min;

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub(crate) fn find_nodes_to_wrap_in_block(
        &self,
        start: usize,
        end: usize,
    ) -> Option<WrapSearchResult> {
        let dom = &self.state.dom;
        let range = dom.find_range(start, end);
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        if leaves.is_empty() {
            None
        } else {
            let first_leaf = leaves.first().unwrap();
            let last_leaf = leaves.last().unwrap();
            let rev_iter = dom.iter_from_handle(&first_leaf.node_handle).rev();
            let iter = dom.iter_from_handle(&last_leaf.node_handle);
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
            // If we have a selection inside a single ListItem we only want to wrap its contents.
            // However, if the selection covers several list items, we should split its parent List
            // and wrap their contents instead.
            let selection_contains_several_list_items = range
                .locations
                .iter()
                .filter(|l| l.kind == ListItem)
                .count()
                > 1;
            for node in rev_iter {
                if !node.is_block_node()
                    && node.kind() != LineBreak
                    && (node.kind() != ListItem
                        || selection_contains_several_list_items)
                {
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

            for node in iter {
                if !node.is_block_node()
                    && node.kind() != LineBreak
                    && (node.kind() != ListItem
                        || selection_contains_several_list_items)
                {
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

            let (first, last) = nodes_to_cover;
            let max_depth = min(first.handle.depth(), last.handle.depth());
            let mut min_depth = 0;
            for i in min_depth..max_depth {
                min_depth = i;
                if first.handle.raw()[i] != last.handle.raw()[i] {
                    break;
                }
            }
            let first_list_item =
                self.state.dom.find_parent_list_item_or_self(&first.handle);
            let last_list_item =
                self.state.dom.find_parent_list_item_or_self(&last.handle);
            if first_list_item.is_some()
                && last_list_item.is_some()
                && first_list_item != last_list_item
                && min_depth > 0
            {
                // We should wrap their parent List instead
                min_depth -= 1;
            }
            // Will wrap an empty text node at the end of the editor
            if first.handle == last.handle && first.kind == LineBreak {
                return None;
            }
            let idx_start = first.handle.raw()[min_depth];
            let idx_end = last.handle.raw()[min_depth];
            let ancestor_handle = first.handle.sub_handle_up_to(min_depth);
            Some(WrapSearchResult {
                ancestor_handle,
                idx_start,
                idx_end,
                range,
            })
        }
    }

    pub(crate) fn find_ancestor_to_split(
        &self,
        handle: &DomHandle,
    ) -> DomHandle {
        if handle.depth() <= 1 {
            DomHandle::root()
        } else {
            for i in (0..handle.depth()).rev() {
                let ancestor_handle = handle.sub_handle_up_to(i);
                let ancestor = self.state.dom.lookup_node(&ancestor_handle);
                if ancestor.is_block_node() || ancestor.kind() == ListItem {
                    return ancestor_handle;
                }
            }
            panic!("Should never reach this point, one of the parents surely can be split.");
        }
    }
}

pub(crate) struct WrapSearchResult {
    pub(crate) ancestor_handle: DomHandle,
    pub(crate) idx_start: usize,
    pub(crate) idx_end: usize,
    pub(crate) range: Range,
}

struct HandleWithKind {
    handle: DomHandle,
    kind: DomNodeKind,
}
