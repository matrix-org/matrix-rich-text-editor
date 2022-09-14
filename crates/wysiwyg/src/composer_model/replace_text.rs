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

use crate::composer_model::base::{slice_from, slice_to};
use crate::dom::nodes::DomNode;
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
                        .find_parent_list_item(&range.node_handle);
                    if let Some(parent_handle) = parent_list_item_handle {
                        self.do_enter_in_list(&parent_handle, e, range)
                    } else {
                        self.do_enter_in_text(
                            &range.node_handle,
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

    fn replace_same_node(&mut self, range: SameNodeRange, new_text: S) {
        // TODO: remove SameNode and NoNode?
        let mut delete_this_node = false;
        let mut add_node_after_this = false;
        let handle = range.node_handle;
        let node = self.state.dom.lookup_node_mut(&handle);
        match node {
            DomNode::Text(ref mut t) => {
                let text = t.data();
                let mut n = slice_to(text, ..range.start_offset);
                n.push_string(&new_text);
                n.push_string(&slice_from(&text, range.end_offset..));
                t.set_data(n);
            }
            DomNode::LineBreak(_) => {
                match (range.start_offset, range.end_offset) {
                    (0, 1) => {
                        // Whole line break is selected, delete it
                        delete_this_node = true;
                    }
                    (1, 1) => {
                        // Cursor is after line break, no need to delete
                    }
                    _ => panic!(
                        "Should not get SameNode range for start of \
                        line break!"
                    ),
                }

                add_node_after_this = true;
                // TODO: create a new text node after this one to contain
                // the contents of new_text
            }
            DomNode::Container(_) => {
                panic!(
                    "Can't deal with ranges containing non-text nodes (yet?)"
                )
            }
        }

        if add_node_after_this {
            match self.state.dom.lookup_node_mut(&handle.parent_handle()) {
                DomNode::Container(parent) => parent.insert_child(
                    handle.index_in_parent() + 1,
                    DomNode::new_text(new_text),
                ),
                _ => panic!("Parent node is not a container!"),
            }
        }

        if delete_this_node {
            match self.state.dom.lookup_node_mut(&handle.parent_handle()) {
                DomNode::Container(parent) => {
                    parent.remove_child(handle.index_in_parent());
                }
                _ => panic!("Parent node is not a container!"),
            }
        }
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
            let mut node = self.state.dom.lookup_node_mut(&loc.node_handle);
            match &mut node {
                DomNode::Container(_) => {
                    // Nothing to do for container nodes
                }
                DomNode::LineBreak(_) => {
                    to_delete.push(loc.node_handle);
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
