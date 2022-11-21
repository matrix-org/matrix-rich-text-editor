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
use crate::dom::nodes::{DomNode, TextNode};
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{DomHandle, DomLocation, Range};
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
            self.backspace_single_cursor(range, e)
        } else {
            self.do_backspace()
        }
    }

    fn backspace_single_cursor(
        &mut self,
        range: Range,
        end_position: usize,
    ) -> ComposerUpdate<S> {
        // Find the first leaf node in this selection - note there
        // should only be one because s == e, so we don't have a
        // selection that spans multiple leaves.
        let first_leaf = range.locations.iter().find(|loc| loc.is_leaf);
        if let Some(leaf) = first_leaf {
            // We are backspacing inside a text node with no
            // selection - we might need special behaviour, if
            // we are at the start of a list item.
            let parent_list_item_handle = self
                .state
                .dom
                .find_parent_list_item_or_self(&leaf.node_handle);
            if let Some(parent_handle) = parent_list_item_handle {
                self.do_backspace_in_list(&parent_handle, end_position)
            } else {
                self.do_backspace()
            }
        } else {
            self.do_backspace()
        }
    }

    /// Remove a single word when user does ctrl/cmd + backspace
    pub fn backspace_word(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();

        // if we're at the start of the string, do nothing
        if s == 0 {
            return ComposerUpdate::keep();
        } 

        // categories of character before cursor
        #[derive(PartialEq)]
        enum Cat {
            Whitespace, 
            Other,
            None
        }
        
        // not at the start of the string from here onwards
        let content = self.state.dom.to_string();
        println!("CONTENT: {}", content);
        let mut search_index = e-1;
        let start_type = match content.chars().nth(e-1) {
            Some(c) => {
                if c.is_whitespace() {
                    Cat::Whitespace
                } else {
                    Cat::Other
                }
            }
            None => Cat::None,
        };
        let mut trigger = start_type.eq(&Cat::Whitespace);

        while search_index > 0 {
            let nth_char = content.chars().nth(search_index);
            match nth_char {
                Some(c) => {
                    if trigger && !c.is_whitespace() {
                        trigger = false
                    }
                    match start_type {
                        Cat::Whitespace => {
                            if c.is_whitespace() && !trigger {
                                break;
                            }
                        },
                        Cat::Other => {
                            if c.is_whitespace() {
                                break;
                            }
                        },
                        Cat::None => break,
                    }
                },
                None => break,
            }
            search_index -= 1; 
        }
        
        self.delete_in(search_index + 1,e)
    }
 
    /// Deletes text in an arbitrary start..end range.
    pub fn delete_in(&mut self, start: usize, end: usize) -> ComposerUpdate<S> {
        self.state.end = Location::from(start);
        self.replace_text_in(S::default(), start, end)
    }

    /// Deletes the character after the current cursor position.
    pub fn delete(&mut self) -> ComposerUpdate<S> {
        if self.state.start == self.state.end {
            let (s, _) = self.safe_selection();
            // If we're dealing with complex graphemes, this value might not be 1
            let next_char_len =
                if let Some((text_node, loc)) = self.get_selected_text_node() {
                    let selection_start_in_str = s - loc.position;
                    Self::find_next_char_len(
                        selection_start_in_str,
                        &text_node.data(),
                    ) as isize
                } else {
                    1
                };
            // Go forward `next_char_len` positions from the current location
            self.state.end += next_char_len;
        }

        self.replace_text(S::default())
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
            let (_, e) = self.safe_selection();
            // If we're dealing with complex graphemes, this value might not be 1
            let prev_char_len =
                if let Some((text_node, loc)) = self.get_selected_text_node() {
                    let selection_end_in_str = e - loc.position;
                    Self::find_previous_char_len(
                        selection_end_in_str,
                        &text_node.data(),
                    ) as isize
                } else {
                    1
                };
            // Go back `prev_char_len` positions from the current location
            self.state.start -= prev_char_len;
        }

        self.replace_text(S::default())
    }

    /// Returns the currently selected TextNode if it's the only leaf node and the cursor is inside
    /// its range.
    fn get_selected_text_node(&self) -> Option<(&TextNode<S>, DomLocation)> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let leaves: Vec<&DomLocation> = range.leaves().collect();
        if s == e && leaves.len() == 1 {
            let leaf = leaves[0];
            if let DomNode::Text(text_node) =
                self.state.dom.lookup_node(&leaf.node_handle)
            {
                return Some((text_node, leaf.clone()));
            }
        }
        None
    }

    /// Returns the length of the [char] for the current [S] string encoding before the given [pos].
    fn find_previous_char_len(pos: usize, str: &S::Str) -> usize {
        let graphemes = str.find_graphemes_at(pos);
        // Take the grapheme before the position
        if let Some(last_grapheme) = graphemes.0 {
            last_grapheme.len()
        } else {
            // Default length for characters
            1
        }
    }

    /// Returns the length of the [char] for the current [S] string encoding after the given [pos].
    fn find_next_char_len(pos: usize, str: &S::Str) -> usize {
        let graphemes = str.find_graphemes_at(pos);
        // Take the grapheme after the position
        if let Some(first_grapheme) = graphemes.1 {
            first_grapheme.len()
        } else {
            // Default length for characters
            1
        }
    }
}
