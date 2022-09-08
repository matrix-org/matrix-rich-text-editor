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

use crate::composer_model::base::{slice, slice_from, slice_to};
use crate::dom::nodes::{ContainerNodeKind, DomNode};
use crate::dom::{DomHandle, MultipleNodesRange, Range, SameNodeRange};
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
            match range {
                Range::SameNode(range) => {
                    let parent_list_item_handle = self
                        .state
                        .dom
                        .find_parent_list_item(range.node_handle.clone());
                    if let Some(parent_handle) = parent_list_item_handle {
                        self.do_enter_in_list(parent_handle, e, range)
                    } else {
                        self.do_enter_in_text(
                            range.node_handle,
                            range.start_offset,
                        )
                    }
                }
                Range::MultipleNodes(_) => {
                    panic!("Unexpected multiple nodes on a 0 length selection")
                }
                Range::NoNode => self.replace_text(S::from_str("\n")),
            }
        } else {
            // Clear selection then enter.
            self.delete();
            self.enter()
        }
    }

    fn do_enter_in_text(
        &mut self,
        handle: DomHandle,
        start: usize,
    ) -> ComposerUpdate<S> {
        let mut added_characters = 0;

        let node = self.state.dom.lookup_node_mut(handle.clone());
        let remaining_text = match node {
            DomNode::Container(_) => panic!("Enter within a non-text node!"),
            DomNode::Text(node) => {
                let before = if start == 0 {
                    added_characters += 1;
                    S::from_str("\u{200b}")
                } else {
                    slice_to(node.data(), ..start)
                };
                let after = slice_from(node.data(), start..);
                node.set_data(before);
                after
            }
        };

        let parent_handle = handle.parent_handle();
        let parent = self.state.dom.lookup_node_mut(parent_handle);
        match parent {
            DomNode::Container(parent) => {
                added_characters += 1;
                // TODO: simpler function for zero-width text
                let mut new_text = S::from_str("\u{200b}");
                new_text.push_string(&remaining_text);
                parent.append_child(DomNode::new_line_break());
                parent.append_child(DomNode::new_text(new_text));
            }
            DomNode::Text(_) => panic!("Parent node was a text node!"),
        }

        self.state.start += added_characters;
        self.state.end = self.state.start;
        self.create_update_replace_all()
    }

    /// Internal: replace some text without modifying the undo/redo state.
    pub(crate) fn do_replace_text_in(
        &mut self,
        new_text: S,
        mut start: usize,
        end: usize,
    ) -> ComposerUpdate<S> {
        let len = new_text.len();

        match self.state.dom.find_range(start, end) {
            Range::SameNode(range) => {
                self.replace_same_node(range, new_text);
            }
            Range::MultipleNodes(range) => {
                self.replace_multiple_nodes(range, new_text)
            }
            Range::NoNode => {
                self.state.dom.append_child(DomNode::new_text(new_text));

                start = 0;
            }
        }

        self.state.start = Location::from(start + len);
        self.state.end = self.state.start;

        // TODO: for now, we replace every time, to check ourselves, but
        // at least some of the time we should not
        self.create_update_replace_all()
    }

    /// Decide whether we need to deleted the previous sibling, and delete it
    /// if so.
    /// Logic: if the text we are replacing starts with a zero-width space, and
    /// this text node is preceded by a br tag, then delete the br tag.
    fn maybe_delete_previous_sibling(
        &mut self,
        text: &S,
        range: &SameNodeRange,
    ) {
        // Check for whether we need to delete a <br /> before us
        if range.start_offset == 0 && range.end_offset > 0 {
            let deleted_text =
                slice(text, range.start_offset..(range.start_offset + 1));
            if deleted_text == S::from_str("\u{200b}") {
                // Delete the node before us
                if range.node_handle.index_in_parent() > 0 {
                    let handle = range.node_handle.prev_sibling();
                    if let DomNode::Container(node) =
                        self.state.dom.lookup_node(handle.clone())
                    {
                        if *node.kind() == ContainerNodeKind::LineBreak {
                            self.state.dom.delete_node(handle);
                        }
                    }
                }
            }
        }
    }

    fn replace_same_node(&mut self, range: SameNodeRange, new_text: S) {
        // TODO: remove SameNode and NoNode?

        let old_text = {
            let node =
                self.state.dom.lookup_node_mut(range.node_handle.clone());
            if let DomNode::Text(ref mut t) = node {
                let text = t.data().clone();

                let mut n = slice_to(&text, ..range.start_offset);
                n.push_string(&new_text);
                n.push_string(&slice_from(&text, range.end_offset..));
                t.set_data(n);
                text
            } else {
                panic!(
                    "Can't deal with ranges containing non-text nodes (yet?)"
                )
            }
        };

        self.maybe_delete_previous_sibling(&old_text, &range)
    }

    fn replace_multiple_nodes(
        &mut self,
        range: MultipleNodesRange,
        new_text: S,
    ) {
        let len = new_text.len();
        let to_delete = self.replace_in_text_nodes(range.clone(), new_text);
        self.delete_nodes(to_delete);

        let pos: usize = self.state.start.into();
        self.join_nodes(&range, pos + len + 1);
    }

    /// Given a range to replace and some new text, modify the nodes in the
    /// range to replace the text with the supplied text.
    /// Returns a list of (handles to) nodes that have become empty and should
    /// be deleted.
    fn replace_in_text_nodes(
        &mut self,
        range: MultipleNodesRange,
        new_text: S,
    ) -> Vec<DomHandle> {
        let mut to_delete = Vec::new();
        let mut first_text_node = true;
        for loc in range.into_iter() {
            let mut node =
                self.state.dom.lookup_node_mut(loc.node_handle.clone());
            match &mut node {
                DomNode::Container(_) => {
                    // Nothing to do for container nodes
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
                            slice_to(old_data, ..loc.start_offset);

                        // and replace with the new content
                        if first_text_node {
                            new_data.push_string(&new_text);
                        }

                        new_data.push_string(&slice_from(
                            old_data,
                            loc.end_offset..,
                        ));
                        node.set_data(new_data);
                    }

                    first_text_node = false;
                }
            }
        }
        to_delete
    }
}
