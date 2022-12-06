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

use crate::dom::nodes::text_node::CharType;
use crate::dom::nodes::{DomNode, TextNode};
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{DomHandle, DomLocation, Range};
use crate::{ComposerModel, ComposerUpdate, Location, UnicodeString};

#[derive(PartialEq, Debug)]
pub enum Direction {
    Forwards,
    Backwards,
}

impl Direction {
    pub fn increment(&self, index: usize) -> usize {
        match self {
            Direction::Forwards => index + 1,
            Direction::Backwards => index - 1,
        }
    }
    pub fn get_index_from_cursor(&self, index: usize) -> usize {
        match self {
            Direction::Forwards => index,
            Direction::Backwards => index - 1,
        }
    }
}

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn backspace(&mut self) -> ComposerUpdate<S> {
        self.push_state_to_history();
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

    /// Deletes the current selection, will return a keep in case where
    /// we don't have a selection
    fn delete_selection(&mut self) -> ComposerUpdate<S> {
        if self.has_cursor() {
            return ComposerUpdate::keep();
        }

        let (s, e) = self.safe_selection();
        self.delete_in(s, e)
    }

    /// Allows deletion between two positions, regardless of argument order
    fn delete_to_cursor(&mut self, position: usize) -> ComposerUpdate<S> {
        if self.has_selection() {
            panic!("Can't delete from a position to a selection")
        }

        let (s, _) = self.safe_selection();

        if s < position {
            self.delete_in(s, position)
        } else {
            self.delete_in(position, s)
        }
    }

    /// Deletes text in an arbitrary start..end range.
    pub fn delete_in(&mut self, start: usize, end: usize) -> ComposerUpdate<S> {
        self.push_state_to_history();
        self.state.end = Location::from(start);
        self.do_replace_text_in(S::default(), start, end)
    }

    /// Deletes the character after the current cursor position.
    pub fn delete(&mut self) -> ComposerUpdate<S> {
        self.push_state_to_history();
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

        self.do_replace_text(S::default())
    }

    /// Remove a single word when user does ctrl/opt + delete
    pub fn delete_word(&mut self) -> ComposerUpdate<S> {
        self.remove_word_in_direction(Direction::Forwards)
    }

    /// Remove a single word when user does ctrl/opt + backspace
    pub fn backspace_word(&mut self) -> ComposerUpdate<S> {
        self.remove_word_in_direction(Direction::Backwards)
    }

    /// Given a direction will get the remove word arguments and then run 'remove_word'
    /// if arguments can be generated
    fn remove_word_in_direction(
        &mut self,
        direction: Direction,
    ) -> ComposerUpdate<S> {
        // if we have a selection, only remove the selection
        if self.has_selection() {
            return self.delete_selection();
        }

        let inital_arguments = self.get_remove_word_arguments(&direction);
        match inital_arguments {
            None => ComposerUpdate::keep(),
            Some(arguments) => {
                // here we have a non-split cursor, a single location, and a textlike node
                let (loc, starting_char_type) = arguments;
                self.remove_word(starting_char_type, direction, loc)
            }
        }
    }

    /// Actually removes words from the dom. Number of arguments is due to adjacent text nodes.
    /// If we can guarantee no adjacent similar nodes, this function could be refactored to just
    /// use start_type, direction and location
    fn remove_word(
        &mut self,
        start_type: CharType,
        direction: Direction,
        location: DomLocation,
    ) -> ComposerUpdate<S> {
        match self.state.dom.lookup_node_mut(&location.node_handle) {
            // we should never be passed a container
            DomNode::Container(_) => ComposerUpdate::keep(),
            // for a linebreak, remove it if we started the operation from the whitespace
            // char type, otherwise keep it
            DomNode::LineBreak(_) => {
                match start_type {
                    CharType::Punctuation | CharType::Other => {
                        ComposerUpdate::keep()
                    }
                    CharType::Whitespace | CharType::Linebreak => self
                        .delete_to_cursor(direction.increment(
                            location.position + location.start_offset,
                        )),
                }
            }
            DomNode::Text(node) => {
                let mut current_offset = location.start_offset;

                // TODO this could be tidied up by making text_node able to
                // generate a directional iterator from a given offset
                // We use unwrap here because we are only ever calling it when
                // definitely inside a text node
                let mut current_type = node
                    .char_type_at_offset(current_offset, &direction)
                    .unwrap();

                while node.offset_is_inside_node(current_offset, &direction)
                    && current_type == start_type
                {
                    let next_offset = direction.increment(current_offset);
                    let next_type = node
                        .char_type_at_offset(current_offset, &direction)
                        .unwrap();

                    if next_type != current_type {
                        break;
                    }

                    current_offset = next_offset;
                    current_type = next_type;
                }

                // we have two scenarios here, we have either stopped looping due to a change of type
                // inside the text node, or we have reached the edge of the text node

                let current_position = location.position + current_offset;
                if node.offset_is_inside_node(current_offset, &direction) {
                    // if we have stopped inside the node, first do the required deletion
                    self.delete_to_cursor(current_position);

                    // if we didn't start this removal at whitespace, stop
                    if start_type != CharType::Whitespace {
                        return ComposerUpdate::keep();
                    }

                    // otherwise, get the next set of arguments and make the recursive call
                    let next_arguments =
                        self.get_remove_word_arguments(&direction);
                    return match next_arguments {
                        None => ComposerUpdate::keep(),
                        Some(arguments) => {
                            let (loc, next_type) = arguments;
                            self.remove_word(
                                next_type, // pass it the new type to remove
                                direction, loc,
                            )
                        }
                    };
                } else {
                    // firstly do the deletion
                    self.delete_to_cursor(current_position);

                    // then if we have reached the end of the dom or the end of a list, stop
                    if self.selection_touches_start_or_end_of_dom(&direction)
                        || location
                            .position_is_end_of_list_item(current_position)
                    {
                        return ComposerUpdate::keep();
                    }

                    // otherwise, make a recursive call to continue to the next node
                    let next_arguments =
                        self.get_remove_word_arguments(&direction);
                    return match next_arguments {
                        None => ComposerUpdate::keep(),
                        Some(details) => {
                            let (_location, _) = details;
                            self.remove_word(
                                start_type, // use the original first type from the remove_word call
                                direction, _location,
                            )
                        }
                    };
                }
            }
        }
    }

    /// In order for the recursive calls to work we need quite a few details
    /// from the cursor location, this gets those details and returns them
    /// as a tuple. Likely this can be replaced by DOM iteration methods
    fn get_remove_word_arguments<'a>(
        &'a mut self,
        direction: &Direction,
    ) -> Option<(DomLocation, CharType)> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let dom_length = self.state.dom.text_len();

        if self.has_selection() {
            return None;
        }

        match range
            .locations
            .iter()
            .find(|loc| {
                // do not include the location if it is at the bad end of the dom
                let exclude_due_to_end_of_dom = match direction {
                    Direction::Forwards => e == dom_length,
                    Direction::Backwards => s == 0,
                };

                let inside_location = loc.start_offset < loc.length;

                let is_correct_boundary_location = match direction {
                    Direction::Forwards => loc.start_offset == 0,
                    Direction::Backwards => loc.start_offset == loc.length,
                };

                // find the first location that we are inside, or if we're at a boundary,
                // choose based on direction
                return !exclude_due_to_end_of_dom
                    && (inside_location || is_correct_boundary_location);
            })
            .cloned()
        {
            None => None,
            Some(location) => {
                let char_type =
                    self.get_char_type_from_location(&location, direction);

                return match char_type {
                    None => None,
                    Some(char_type) => Some((location, char_type)),
                };
            }
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
        let first_leaf = range.locations.iter().find(|loc| loc.is_leaf());
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

    /// Removes the node at [cur_handle] and then will recursively delete any empty parent nodes
    /// until we reach the [top_handle] node.
    pub(crate) fn remove_and_clean_up_empty_nodes_until(
        &mut self,
        cur_handle: &DomHandle,
        top_handle: &DomHandle,
    ) {
        self.state.dom.remove(cur_handle);
        if cur_handle != top_handle
            && self.state.dom.parent(cur_handle).children().is_empty()
        {
            self.remove_and_clean_up_empty_nodes_until(
                &cur_handle.parent_handle(),
                top_handle,
            )
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

        self.do_replace_text(S::default())
    }

    /// Given a node, return the type of character adjacent to the cursor offset position
    /// bearing in mind the direction
    fn get_char_type_from_location(
        &self,
        location: &DomLocation,
        direction: &Direction,
    ) -> Option<CharType> {
        let node = self.state.dom.lookup_node(&location.node_handle);
        match node {
            DomNode::Container(_) => {
                // we should never get a container type!
                panic!("hit container in get_details_from_range_and_direction")
            }
            DomNode::LineBreak(_) => {
                // we have to treat linebreaks as chars, so they have their own CharType
                Some(CharType::Linebreak)
            }
            DomNode::Text(text_node) => {
                text_node.char_type_at_offset(location.start_offset, direction)
            }
        }
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
