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

use crate::composer_model::base::adjust_handles_for_delete;
use crate::dom::nodes::DomNode;
use crate::dom::{DomHandle, Range};
use crate::{ComposerModel, ComposerUpdate, Location, UnicodeString};

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn backspace(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();

        if s == e {
            // We have no selection - check for special list behaviour
            // TODO: should probably also get inside here if our selection
            // only contains a zero-wdith space.
            let range = self.state.dom.find_range(s, e);
            match range {
                Range::SameNode(range) => {
                    let mrange =
                        self.state.dom.convert_same_node_range_to_multi(range);
                    // Find the first leaf node in this selection - note there
                    // should only be one because s == e, so we don't have a
                    // selection that spans multiple leaves.
                    let first_leaf =
                        mrange.locations.iter().find(|loc| loc.is_leaf);
                    if let Some(leaf) = first_leaf {
                        // We are backspacing inside a text node with no
                        // selection - we might need special behaviour, if
                        // we are at the start of a list item.
                        let parent_list_item_handle = self
                            .state
                            .dom
                            .find_parent_list_item_or_self(&leaf.node_handle);
                        if let Some(parent_handle) = parent_list_item_handle {
                            self.do_backspace_in_list(&parent_handle, e)
                        } else {
                            self.do_backspace()
                        }
                    } else {
                        self.do_backspace()
                    }
                }
                _ => panic!("s == e, so this will always be SameNode!"),
            }
        } else {
            self.do_backspace()
        }
    }

    /// Deletes text in an arbitrary start..end range.
    pub fn delete_in(&mut self, start: usize, end: usize) -> ComposerUpdate<S> {
        self.state.end = Location::from(start);
        self.replace_text_in(S::new(), start, end)
    }

    /// Deletes the character after the current cursor position.
    pub fn delete(&mut self) -> ComposerUpdate<S> {
        if self.state.start == self.state.end {
            // Go forward 1 from the current location
            self.state.end += 1;
        }

        self.replace_text(S::new())
    }

    pub(crate) fn delete_nodes(&mut self, mut to_delete: Vec<DomHandle>) {
        // Delete in reverse order to avoid invalidating handles
        to_delete.reverse();

        // We repeatedly delete to ensure anything that became empty because
        // of deletions is itself deleted.
        while !to_delete.is_empty() {
            // Keep a list of things we will delete next time around the loop
            let mut new_to_delete = Vec::new();

            for handle in to_delete.into_iter() {
                let child_index =
                    handle.raw().last().expect("Text node can't be root!");
                let parent_handle = handle.parent_handle();
                let mut parent = self.state.dom.lookup_node_mut(&parent_handle);
                match &mut parent {
                    DomNode::Container(parent) => {
                        parent.remove_child(*child_index);
                        adjust_handles_for_delete(&mut new_to_delete, &handle);
                        if parent.children().is_empty() {
                            new_to_delete.push(parent_handle);
                        }
                    }
                    _ => {
                        panic!("Parent must be a container!");
                    }
                }
            }

            to_delete = new_to_delete;
        }
    }

    pub(crate) fn do_backspace(&mut self) -> ComposerUpdate<S> {
        if self.state.start == self.state.end {
            // Go back 1 from the current location
            self.state.start -= 1;
        }

        self.replace_text(S::new())
    }
}
