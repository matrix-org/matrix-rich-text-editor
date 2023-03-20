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

use crate::dom::nodes::dom_node::DomNodeKind::{Link, ListItem};
use crate::dom::nodes::text_node::CharType;
use crate::dom::nodes::{DomNode, TextNode};
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{DomHandle, DomLocation, Range};
use crate::{ComposerModel, ComposerUpdate, Location, UnicodeString};

#[derive(PartialEq, Eq, Debug)]
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
        self.handle_non_editable_selection();

        let (s, e) = self.safe_selection();
        if s == e {
            // We have no selection - check for special list behaviour
            // TODO: should probably also get inside here if our selection
            // only contains a zero-wdith space.
            let range = self.state.dom.find_range(s, e);
            self.backspace_single_cursor(range)
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

    /// If we have cursor at the edge of or inside a non-editable text node, expand the selection to cover
    /// the whole of that node before continuing with the backspace/deletion flow
    fn handle_non_editable_selection(&mut self) {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);

        let first_leaf = range.locations.iter().find(|loc| {
            loc.is_leaf() || (loc.kind.is_block_kind() && loc.is_empty())
        });
        if let Some(leaf) = first_leaf {
            let parent_link_loc =
                range.deepest_node_of_kind(Link, Some(&leaf.node_handle));
            if let Some(link) = parent_link_loc {
                if self
                    .state
                    .dom
                    .lookup_container(&link.node_handle)
                    .is_immutable_link()
                {
                    self.select(
                        Location::from(link.position),
                        Location::from(link.position + link.length),
                    );
                }
            }
        }
    }

    /// Deletes the character after the current cursor position.
    pub fn delete(&mut self) -> ComposerUpdate<S> {
        self.push_state_to_history();

        self.handle_non_editable_selection();

        if self.state.start == self.state.end {
            let (s, _) = self.safe_selection();
            // If we're dealing with complex graphemes, this value might not be 1
            let next_char_len =
                if let Some((text_node, loc)) = self.get_selected_text_node() {
                    let selection_start_in_str = s - loc.position;
                    Self::find_next_char_len(
                        selection_start_in_str,
                        text_node.data(),
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

        let args = self.get_remove_word_arguments(&direction);
        match args {
            None => ComposerUpdate::keep(),
            Some(arguments) => {
                // here we have a non-split cursor, a single location, and a textlike node
                let (location, start_type) = arguments;
                self.remove_word(start_type, direction, location)
            }
        }
    }

    /// Remove the word runs from the dom and can recursively call itself
    fn remove_word(
        &mut self,
        start_type: CharType,
        direction: Direction,
        location: DomLocation,
    ) -> ComposerUpdate<S> {
        match self.state.dom.lookup_node_mut(&location.node_handle) {
            // we should never be passed a container
            DomNode::Container(_) => ComposerUpdate::keep(),
            DomNode::LineBreak(_) => {
                // for a linebreak, remove it if we started the operation from the whitespace
                // char type, otherwise keep it
                match start_type {
                    CharType::Whitespace => self.delete_to_cursor(
                        direction.increment(location.index_in_dom()),
                    ),
                    _ => ComposerUpdate::keep(),
                }
            }
            DomNode::Text(node) => {
                // we are guaranteed to get valid chars here, so can use unwrap
                let mut current_offset = location.start_offset;
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

                // determine our current position in the dom
                let current_position = location.position + current_offset;
                let offset_is_inside_node =
                    node.offset_is_inside_node(current_offset, &direction);

                // delete to the cursor
                self.delete_to_cursor(current_position);

                // if we have stopped inside the node and we didn't start at whitespace, stop
                if offset_is_inside_node && start_type != CharType::Whitespace {
                    return ComposerUpdate::keep();
                }

                // otherwise make a recursive call
                let next_args = self.get_remove_word_arguments(&direction);
                match next_args {
                    None => ComposerUpdate::keep(),
                    Some(args) => {
                        let (location, next_type) = args;
                        let type_argument = if offset_is_inside_node {
                            // where we finished inside the node, get the next type when we
                            // are making a recursive call after having removed whitespace
                            next_type
                        } else {
                            // if we hit the edge of the node, we need to make the next call
                            // with the same initial starting type
                            start_type
                        };

                        self.remove_word(type_argument, direction, location)
                    }
                }
            }
        }
    }

    /// In order for the recursive calls to work we need quite a few details
    /// from the cursor location, this gets those details and returns them
    /// as a tuple.
    fn get_remove_word_arguments(
        &self,
        direction: &Direction,
    ) -> Option<(DomLocation, CharType)> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        let dom_length = self.state.dom.text_len();

        if self.has_selection() {
            return None;
        }

        let selected_location = range
            .locations
            .iter()
            .find(|loc| {
                // do not include the location if it is at the wrong end of the dom
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
                !exclude_due_to_end_of_dom
                    && (inside_location || is_correct_boundary_location)
            })
            .cloned();
        match selected_location {
            None => None,
            Some(location) => {
                let char_type =
                    self.get_char_type_from_location(&location, direction);
                char_type.map(|char_type| (location, char_type))
            }
        }
    }

    /// Given a location, return the type of character adjacent to the cursor offset position
    /// bearing in mind the direction
    fn get_char_type_from_location(
        &self,
        location: &DomLocation,
        direction: &Direction,
    ) -> Option<CharType> {
        let node = self.state.dom.lookup_node(&location.node_handle);
        match node {
            DomNode::Container(_) => None,
            DomNode::LineBreak(_) => {
                // we have to treat linebreaks as chars, this type fits best
                Some(CharType::Whitespace)
            }
            DomNode::Text(text_node) => {
                text_node.char_type_at_offset(location.start_offset, direction)
            }
        }
    }

    fn backspace_single_cursor(&mut self, range: Range) -> ComposerUpdate<S> {
        // Find the first leaf node in this selection - note there
        // should only be one because s == e, so we don't have a
        // selection that spans multiple leaves.
        let first_leaf = range.locations.iter().find(|loc| {
            loc.is_leaf() || (loc.kind.is_block_kind() && loc.is_empty())
        });
        if let Some(leaf) = first_leaf {
            // We are backspacing inside a text node with cursor
            // selection - we might need special behaviour, if
            // we are at the start of a list item.
            let parent_list_item_loc =
                range.deepest_node_of_kind(ListItem, Some(&leaf.node_handle));
            if let Some(list_item_loc) = parent_list_item_loc {
                if list_item_loc.start_offset == 0 {
                    self.do_backspace_in_list(&list_item_loc.node_handle)
                } else {
                    self.do_backspace()
                }
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
                        text_node.data(),
                    ) as isize
                } else {
                    1
                };
            // Go back `prev_char_len` positions from the current location
            self.state.start -= prev_char_len;
        }

        self.do_replace_text(S::default())
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
