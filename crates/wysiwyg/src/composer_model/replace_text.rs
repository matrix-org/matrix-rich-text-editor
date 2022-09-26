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

use crate::dom::nodes::text_node::ZWSP;
use crate::dom::nodes::DomNode;
use crate::dom::unicode_string::UnicodeStringExt;
use crate::dom::{DomHandle, DomLocation, Range};
use crate::{ComposerModel, ComposerUpdate, Location, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    /// Replaces text in the current selection with new_text.
    /// Treats its input as plain text, so any HTML code will show up in
    /// the document (i.e. it will be escaped).
    pub fn replace_text(&mut self, new_text: S) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        self.replace_text_in(new_text, s, e)
    }

    /// Replaces text in the an arbitrary start..end range with new_text.
    pub fn replace_text_in(
        &mut self,
        new_text: S,
        start: usize,
        end: usize,
    ) -> ComposerUpdate<S> {
        // Store current Dom
        self.push_state_to_history();
        self.do_replace_text_in(new_text, start, end)
    }

    pub fn enter(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();

        if s == e {
            let range = self.state.dom.find_range(s, e);
            self.enter_with_zero_length_selection(range)
        } else {
            // Clear selection then enter.
            // TODO: adds an extra entry to the undo log, I think.
            self.delete();
            self.enter()
        }
    }

    fn enter_with_zero_length_selection(
        &mut self,
        range: Range,
    ) -> ComposerUpdate<S> {
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        if leaves.len() == 1 {
            let location = leaves[0];
            let handle = &location.node_handle;
            let parent_list_item_handle =
                self.state.dom.find_parent_list_item_or_self(handle);
            if let Some(parent_list_item_handle) = parent_list_item_handle {
                self.do_enter_in_list(
                    &parent_list_item_handle,
                    location.position + location.start_offset,
                    handle,
                    location.start_offset,
                    location.end_offset,
                )
            } else {
                self.do_enter_in_text(handle, location.start_offset)
            }
        } else if leaves.len() == 0 {
            // Selection doesn't contain any text node. We can assume it's an empty Dom.
            self.state
                .dom
                .document_mut()
                .append_child(DomNode::new_line_break());
            self.state.start += 1;
            self.state.end = self.state.start;
            self.create_update_replace_all()
        } else {
            panic!("Unexpected multiple nodes on a 0 length selection")
        }
    }

    /// Internal: replace some text without modifying the undo/redo state.
    pub(crate) fn do_replace_text_in(
        &mut self,
        new_text: S,
        mut start: usize,
        end: usize,
    ) -> ComposerUpdate<S> {
        let len = new_text.len();

        let range = self.state.dom.find_range(start, end);
        if range.is_empty() {
            start = 0;
            self.state.dom.append_child(DomNode::new_text(new_text));
        } else {
            let start_offset = self.replace_multiple_nodes(range, new_text);
            start -= start_offset
        }

        self.state.start = Location::from(start + len);
        self.state.end = self.state.start;

        // TODO: for now, we replace every time, to check ourselves, but
        // at least some of the time we should not
        self.create_update_replace_all()
    }

    fn replace_multiple_nodes(&mut self, range: Range, new_text: S) -> usize {
        let len = new_text.len();
        let (to_add, to_delete, start_offset) =
            self.replace_in_text_nodes(range.clone(), new_text);

        // We only add nodes in one special case: when the selection ends at
        // a BR tag. In that case, the only nodes that might be deleted are
        // going to be before the one we add here, so their handles won't be
        // invalidated by the add we do here.
        for (parent_handle, idx, node) in to_add.into_iter().rev() {
            let parent = self.state.dom.lookup_node_mut(&parent_handle);
            if let DomNode::Container(parent) = parent {
                parent.insert_child(idx, node);
            } else {
                panic!("Parent was not a container!");
            }
        }

        // Delete the nodes marked for deletion
        self.delete_nodes(to_delete);

        // If our range covered multiple text-like nodes, join together
        // the two sides of the range.
        if range.leaves().count() > 1 {
            // Calculate the position 1 code unit after the end of the range,
            // after the in-between characters have been deleted, and the new
            // characters have been inserted.
            let new_pos = range.start() + len + 1;

            // Note: the handles in range may have been made invalid by deleting
            // nodes above, but the first text node in it should not have been
            // invalidated, because it should not have been deleted.
            self.join_nodes(&range, new_pos);
        }

        start_offset
    }

    /// Given a range to replace and some new text, modify the nodes in the
    /// range to replace the text with the supplied text.
    /// Returns:
    /// * a list of nodes to create (parent_handle, index, node), and
    /// * a list of (handles to) nodes that have become empty and should
    ///   be deleted.
    /// * an offset to be removed from the start selection
    /// NOTE: all nodes to be created are later in the Dom than all nodes to
    /// be deleted, so you can safely add them before performing the
    /// deletions, and the handles of the deletions will remain valid.
    fn replace_in_text_nodes(
        &mut self,
        range: Range,
        new_text: S,
    ) -> (Vec<(DomHandle, usize, DomNode<S>)>, Vec<DomHandle>, usize) {
        let mut start_offset = 0;
        let mut to_delete = Vec::new();
        let mut to_add = Vec::new();
        let mut first_text_node = true;

        let start = range.start();
        let end = range.end();

        for loc in range.into_iter() {
            let parent_handle = loc.node_handle.parent_handle();
            let parent_is_list_item = if let DomNode::Container(n) =
                self.state.dom.lookup_node(&parent_handle)
            {
                n.is_list_item()
            } else {
                false
            };

            let is_last_item_in_parent = if let DomNode::Container(n) =
                self.state.dom.lookup_node(&parent_handle)
            {
                loc.node_handle.index_in_parent() + 1 == n.children().len()
            } else {
                false
            };

            let mut node = self.state.dom.lookup_node_mut(&loc.node_handle);
            match &mut node {
                DomNode::Container(_) => {
                    // Nothing to do for container nodes
                }
                DomNode::LineBreak(_) => {
                    match (loc.start_offset, loc.end_offset) {
                        (0, 1) => {
                            // Whole line break is selected, delete it
                            to_delete.push(loc.node_handle.clone());
                        }
                        (1, 1) => {
                            // Cursor is after line break, no need to delete
                        }
                        _ => panic!(
                            "Should not get a range at start of a line break!"
                        ),
                    }
                    if start >= loc.position && end == loc.position + 1 {
                        // NOTE: if you add something else to `to_add` you will
                        // probably break our assumptions in the method that
                        // calls this one!
                        // We are assuming we only add nodes AFTER all the
                        // deleted nodes. (That is true in this case, because
                        // we are checking that the selection ends inside this
                        // line break.)

                        // No need to add an empty TextNode as there is already some other node
                        if new_text.is_empty() && !is_last_item_in_parent {
                            break;
                        }

                        let node = if new_text.is_empty() {
                            DomNode::new_zwsp()
                        } else {
                            DomNode::new_text(new_text.clone())
                        };
                        to_add.push((
                            loc.node_handle.parent_handle(),
                            loc.node_handle.index_in_parent() + 1,
                            node,
                        ));
                    }
                }
                DomNode::Text(node) => {
                    let old_data = node.data();

                    // If this is not the first node, and the selections spans
                    // it, delete it.
                    if loc.start_offset == 0
                        && loc.end_offset == old_data.len()
                        && !first_text_node
                    {
                        to_delete.push(loc.node_handle);
                    } else {
                        // Otherwise, delete the selected text
                        let mut new_data =
                            old_data[..loc.start_offset].to_owned();

                        if new_data == ZWSP.into() && !parent_is_list_item {
                            new_data = S::default();
                            // TODO: check if this can be avoided
                            start_offset += 1;
                        }

                        // and replace with the new content
                        if first_text_node {
                            new_data.push(new_text.deref());
                        }

                        new_data.push(&old_data[loc.end_offset..]);
                        node.set_data(new_data);
                    }

                    first_text_node = false;
                }
            }
        }
        (to_add, to_delete, start_offset)
    }
}
