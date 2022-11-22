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

// categories of character
#[derive(PartialEq)]
#[derive(Debug)]
enum Cat {
    Whitespace,
    Newline,
    Punctuation,
    Other,
    None
}

#[derive(PartialEq)]
#[derive(Debug)]
enum Dir {
    Forwards,
    Backwards
}

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

        // if we have a selection, only remove the selection 
        if s != e {
            return self.delete_in(s, e);
        }

        // if we're at the start of the string, do nothing
        if s == 0 {
            return ComposerUpdate::keep();
        }
        
        // not at the start of the string from here onwards
        let content = self.state.dom.to_string();
        println!("CONTENT: {}", content);
        let start_type = self.get_char_type_at(e-1);

        // next actions depend on start type
        match start_type {
            Cat::Whitespace => {
                let (ws_delete_index, stopped_at_newline) = self.get_end_index_of_run(e-1, &Dir::Backwards);
                println!("stopped at newline! index: {}", ws_delete_index);
                if stopped_at_newline {
                    // -1 to account for the fact we want to remove the newline
                    return self.delete_in(ws_delete_index - 1, e);
                } else {
                    self.delete_in(ws_delete_index, e);
                    let (_s, _e) = self.safe_selection();
                    let (non_ws_delete_index, _) = self.get_end_index_of_run(_e-1,&Dir::Backwards);
                    return self.delete_in(non_ws_delete_index, _e);
                }
            },
            Cat::Newline => {
                return self.delete_in(s-1, e);
            },
            Cat::Punctuation => {
                let (delete_index, _) = self.get_end_index_of_run(e-1,&Dir::Backwards);
                return self.delete_in(delete_index, e);
            },
            Cat::Other => {
                let (delete_index, _) = self.get_end_index_of_run(e-1,&Dir::Backwards);
                println!("DELETE INDEX: {delete_index}");
                return self.delete_in(delete_index, e);
            }
            Cat::None => ComposerUpdate::keep(),
        }
    }

    // types defined in the Cat struct
    fn get_char_type_at(& self, index: usize) -> Cat {
        let content = self.state.dom.to_string();
        match content.chars().nth(index) {
            Some(c) => {
                // handle newlines separately, otherwise they'll get classed as white space
                if c == '\n' {
                    return Cat::Newline;
                }

                if c.is_whitespace() {
                    return Cat::Whitespace;
                } else if c.is_ascii_punctuation() || c == '£'{
                    // is_ascii_punctuation doesn't include £, if we don't add this, behaviour will
                    // be as per google docs
                    return Cat::Punctuation;
                } else {
                    return Cat::Other;
                }
            },
            None => Cat::None,
        }
    }

    // figure out where the run ends and also if we're returning due to a 
    // newline (true) or a change in character type (false)
    fn get_end_index_of_run(& self, start: usize, direction: &Dir) -> (usize, bool) {
        let start_type = self.get_char_type_at(start);
        let mut current_index = start.clone();
        let mut current_type = self.get_char_type_at(current_index);
        let mut stopped_at_newline = start_type.eq(&Cat::Newline);
        let mut would_hit_end = false;

        fn increment(index: usize, dir: &Dir) -> usize {
            match dir {
                Dir::Backwards => index - 1,
                Dir::Forwards => index + 1
            }
        }
        fn decrement(index: usize, dir: &Dir) -> usize {
            match dir {
                Dir::Backwards => index + 1,
                Dir::Forwards => index - 1
            }
        }

        fn check_condition(index: usize, max: usize, start_type: &Cat, current_type: &Cat, dir: &Dir, stopped_at_newline: bool) -> bool {
            let base_condition = current_type.eq(start_type) && !stopped_at_newline;
            match dir {
                Dir::Backwards => base_condition && index > 0,
                Dir::Forwards => base_condition && index < max
            }
        }

        while check_condition(current_index, self.state.dom.text_len(), &start_type, &current_type, direction, stopped_at_newline) {
            current_index = increment(current_index, direction);
            current_type = self.get_char_type_at(current_index);
            if current_type.eq(&start_type) && (current_index == 0 || current_index == self.state.dom.text_len()) {
                would_hit_end = true;
            }
            if current_type.eq(&Cat::Newline) {
                stopped_at_newline = true;
            }
        }

        // if we'd have hit the end, return that index, otherwise
        // return the index of the end of the run
        match would_hit_end {
            true => (current_index, stopped_at_newline),
            false => (decrement(current_index, direction), stopped_at_newline),
        }
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

    /// Remove a single word when user does ctrl/cmd + delete
    pub fn delete_word(&mut self) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();

        // if we have a selection, only remove the selection 
        if s != e {
            return self.delete_in(s, e);
        }

        // if we're at the end of the string, do nothing
        if e == self.state.dom.text_len() {
            return ComposerUpdate::keep();
        }
        
        // not at the start of the string from here onwards
        let content = self.state.dom.to_string();
        println!("CONTENT:{}", content);
        let start_type = self.get_char_type_at(e);
        println!("start type - {:?}", start_type);

        // next actions depend on start type
        match start_type {
            Cat::Whitespace => {
                let (ws_delete_index, stopped_at_newline) = self.get_end_index_of_run(e,&Dir::Forwards);
                println!("stopped at index: {}", ws_delete_index);
                if stopped_at_newline {
                    println!("stopped at newline! index: {}", ws_delete_index);
                    // +2 to account for the fact we want to remove the newline
                    return self.delete_in(s, ws_delete_index + 2);
                } else {
                    self.delete_in(s, ws_delete_index + 1);
                    let (_s, _e) = self.safe_selection();
                    println!("s - {_s} and e - {_e}"); 
                    println!("NEW CONTENT:{}", self.state.dom.to_string()); 
                    let (non_ws_delete_index, _) = self.get_end_index_of_run(_e+1,&Dir::Forwards);
                    return self.delete_in(_s, non_ws_delete_index + 1);
                }
            },
            Cat::Newline => {
                return self.delete_in(s, e+1);
            },
            Cat::Punctuation => {
                let (delete_index, _) = self.get_end_index_of_run(e+1,&Dir::Forwards);
                println!("think we should delete punctutation from {delete_index}");
                return self.delete_in(s, delete_index + 1);
            },
            Cat::Other => {
                let (delete_index, _) = self.get_end_index_of_run(e+1,&Dir::Forwards);
                println!("think we should delete to {delete_index}");
                return self.delete_in(s, delete_index + 1);
            }
            Cat::None => ComposerUpdate::keep(),
        }
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
