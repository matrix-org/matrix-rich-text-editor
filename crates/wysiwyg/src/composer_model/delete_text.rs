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

use core::panic;

use crate::dom::nodes::dom_node::DomNodeKind;
use crate::dom::nodes::{ContainerNode, DomNode, TextNode};
use crate::dom::unicode_string::{UnicodeStr, UnicodeStrExt};
use crate::dom::{DomHandle, DomLocation, Range};
use crate::{ComposerModel, ComposerUpdate, Location, UnicodeString};

// categories of character
#[derive(PartialEq, Debug)]
enum CharType {
    Whitespace,
    Linebreak,
    Punctuation,
    Other,
}

#[derive(PartialEq, Debug)]
enum Direction {
    Forwards,
    Backwards,
}

impl Direction {
    fn increment(&self, index: usize) -> usize {
        match self {
            Direction::Forwards => index + 1,
            Direction::Backwards => index - 1,
        }
    }
    fn get_index_from_cursor(&self, index: usize) -> usize {
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
            // We have no selection - check for special list behaviour`
            // TODO: should probably also get inside here if our selection
            // only contains a zero-wdith space.
            let range = self.state.dom.find_range(s, e);
            self.backspace_single_cursor(range, e)
        } else {
            self.do_backspace()
        }
    }

    /// Allows deletion between two positions, regardless of argument order
    fn delete_between(
        &mut self,
        pos1: usize,
        pos2: usize,
    ) -> ComposerUpdate<S> {
        if pos1 < pos2 {
            self.delete_in(pos1, pos2)
        } else {
            self.delete_in(pos2, pos1)
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
        self.remove_word_in_direction(&Direction::Forwards)
    }

    /// Remove a single word when user does ctrl/opt + backspace
    pub fn backspace_word(&mut self) -> ComposerUpdate<S> {
        self.remove_word_in_direction(&Direction::Backwards)
    }

    /// Given a direction and a range will run the remove word method
    fn remove_word_in_direction(
        &mut self,
        dir: &Direction,
    ) -> ComposerUpdate<S> {
        // start off by getting the current actual cursor and making a range
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);

        // if we have a selection, only remove the selection
        if s != e {
            return self.delete_in(s, e);
        }

        // to remove a word, we need details from the current range
        let current_details =
            self.get_details_from_range_and_direction(range, dir);
        match current_details {
            None => ComposerUpdate::keep(),
            Some(details) => {
                // here we have a non-split cursor, a single location, and a textlike node
                // we need to track the individual node start/end/offset due to
                // adjacent similar nodes
                let (
                    loc,
                    node_handle,
                    starting_char_type,
                    node_start,
                    node_offset,
                ) = details;
                self.remove_word(
                    s,
                    starting_char_type,
                    dir,
                    loc,
                    node_handle,
                    node_start,
                    node_offset,
                )
            }
        }
    }

    // Actually removes words from the dom
    fn remove_word(
        &mut self,
        start_position: usize,
        start_type: CharType,
        dir: &Direction,
        location: DomLocation,
        node_handle: DomHandle,
        node_start: usize,
        node_offset: usize,
    ) -> ComposerUpdate<S> {
        let node = self.state.dom.lookup_node_mut(&node_handle);
        let is_list_item = node.is_list_item();
        match node {
            // we should never be passed a container
            DomNode::Container(_) => ComposerUpdate::keep(),
            // for a linebreak, remove it if we started the operation from the whitespace
            // char type, otherwise keep it
            DomNode::LineBreak(_) => match start_type {
                CharType::Punctuation | CharType::Other => {
                    ComposerUpdate::keep()
                }
                CharType::Whitespace | CharType::Linebreak => self
                    .delete_between(
                        start_position,
                        dir.increment(start_position),
                    ),
            },
            DomNode::Text(text_node) => {
                let node_length = text_node.data().len();
                let mut current_offset = node_offset;

                // we can guarantee that we're inside the text node and stop when we hit
                // the edge of the node, so use unwrap to get the types
                let mut current_type = get_char_type_from_text_node(
                    text_node,
                    current_offset,
                    dir,
                )
                .unwrap();

                while offset_is_inside_node(current_offset, node_length, dir)
                    && current_type.eq(&start_type)
                {
                    let next_offset = dir.increment(current_offset);
                    let next_type = get_char_type_from_text_node(
                        text_node,
                        current_offset,
                        dir,
                    )
                    .unwrap();

                    if !next_type.eq(&current_type) {
                        break;
                    }

                    current_offset = next_offset;
                    current_type = next_type;
                }

                let current_position = node_start + current_offset;

                if offset_is_inside_node(current_offset, node_length, dir) {
                    if start_type.eq(&CharType::Whitespace) {
                        // for whitespace, we remove that run and then make a recursive
                        // call to also remove the next run
                        self.delete_between(start_position, current_position);

                        let (_s, _e) = self.safe_selection();
                        let _range = self.state.dom.find_range(_s, _e);
                        let next_details = self
                            .get_details_from_range_and_direction(_range, dir);
                        return match next_details {
                            None => ComposerUpdate::keep(),
                            Some(details) => {
                                let (
                                    loc,
                                    node_handle,
                                    next_type,
                                    node_start,
                                    node_offset,
                                ) = details;
                                self.remove_word(
                                    _s,
                                    next_type, // pass it the new type to remove
                                    dir,
                                    loc,
                                    node_handle,
                                    node_start,
                                    node_offset,
                                )
                            }
                        };
                    }
                    return self
                        .delete_between(start_position, current_position);
                } else {
                    let reached_end_of_dom = match dir {
                        Direction::Forwards => {
                            current_position == self.state.dom.text_len()
                        }
                        Direction::Backwards => current_position == 0,
                    };

                    let reached_end_of_list_item = is_list_item
                        && (current_position == location.position
                            || current_position
                                == location.position + location.length);

                    // if we have reached the end of the dom or the end of a list item delete then stop
                    if reached_end_of_dom || reached_end_of_list_item {
                        return self
                            .delete_between(start_position, current_position);
                    }

                    // otherwise we delete then make the recursive call
                    self.delete_between(start_position, current_position);

                    let (_s, _e) = self.safe_selection();
                    let _range = self.state.dom.find_range(_s, _e);
                    let next_details =
                        self.get_details_from_range_and_direction(_range, dir);
                    // the issue here is pretty simple, it's how do we get to the next text node properly
                    return match next_details {
                        None => ComposerUpdate::keep(),
                        Some(details) => {
                            let (loc, node_handle, _, node_start, node_offset) =
                                details;
                            self.remove_word(
                                _s,
                                start_type, // nb using the original first type from the remove_word call
                                dir,
                                loc,
                                node_handle,
                                node_start,
                                node_offset,
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
    fn get_details_from_range_and_direction<'a>(
        &'a mut self,
        range: Range,
        dir: &Direction,
    ) -> Option<(DomLocation, DomHandle, CharType, usize, usize)> {
        // check that we're not trying to move outside the dom
        let trying_to_move_outside_dom = match dir {
            Direction::Forwards => range.start() == self.state.dom.text_len(),
            Direction::Backwards => range.start() == 0,
        };

        if trying_to_move_outside_dom {
            return None;
        }

        let position = range.start();

        match self.get_location_from_range(range, dir) {
            None => {
                // if we can't get a location, we're not going to be able to do
                // anything so return
                None
            }
            Some(loc) => {
                // we now have a single location, so find the first text like node,
                // noting that due to adjacent similar nodes we have to track individual
                // node start,end and offset positions
                let text_node_details = self
                    .get_text_like_node_details_from_location(
                        &loc, dir, position,
                    );

                match text_node_details {
                    None => {
                        // if we haven't found a text node, move the cursor to
                        // one end of the location and recurse self
                        let next_cursor_position = match dir {
                            Direction::Forwards => loc.position + loc.length,
                            Direction::Backwards => loc.position,
                        };
                        let next_range = self.state.dom.find_range(
                            next_cursor_position,
                            next_cursor_position,
                        );
                        self.get_details_from_range_and_direction(
                            next_range, dir,
                        )
                    }
                    Some((node_handle, node_start, node_offset)) => {
                        let node = self.state.dom.lookup_node(&node_handle);
                        let char_type =
                            get_char_type_from_node(node, node_offset, dir);

                        match char_type {
                            None => {
                                println!("couldn't find the char type");
                                // if we haven't got a char type we have tried looking outside a text node,
                                // so will need to recurse BUT where do we check for the dom edge??
                                // EITHER panic or recurse as per the above TODO
                                // I think we should go to the next
                                None
                            }
                            Some(char_type) => {
                                // we finally have all the stuff we need
                                Some((
                                    loc.clone(),
                                    node_handle,
                                    char_type,
                                    node_start,
                                    node_offset,
                                ))
                            }
                        }
                    }
                }
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

    /// Given a location, find the handle and start/end/offset positions for the
    /// node. This is required to allow us to cope with similar adjacent nodes
    fn get_text_like_node_details_from_location(
        &self,
        location: &DomLocation,
        direction: &Direction,
        position: usize,
    ) -> Option<(DomHandle, usize, usize)> {
        // if we find a text or linebreak node, return them with their details
        // if we find a container node, recursively search it
        match self.state.dom.lookup_node(&location.node_handle) {
            DomNode::Container(node) => recursively_search_container(
                node, direction, position, location,
            ),
            DomNode::Text(_) | DomNode::LineBreak(_) => Some((
                location.node_handle.clone(),
                location.position,
                location.start_offset,
            )),
        }
    }

    /// Given a range and direction, filter the locations and then return a single location if found
    fn get_location_from_range<'a>(
        &'a mut self,
        range: Range,
        direction: &Direction,
    ) -> Option<DomLocation> {
        // TODO this is required because "<br />|<br >" only returns a single location, for the
        // tag to the left of the cursor, if we got both tags' locations we could remove this block
        if range.locations.len() == 1 {
            return range.locations.into_iter().nth(0);
        }

        return range
            .locations
            .iter()
            // zero length locations cause issues, so filter them out
            .filter(|loc| loc.length != 0)
            .find(|loc| {
                // find the first location that we are inside, or if we're at a boundary,
                // choose based on direction
                return loc.start_offset < loc.length
                    || match direction {
                        Direction::Forwards => loc.start_offset == 0,
                        Direction::Backwards => loc.start_offset == loc.length,
                    };
            })
            .cloned();
    }

    /// Deletes the given [to_delete] nodes and then removes any given parent nodes that became
    /// empty, recursively.
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
                let parent = self.state.dom.parent_mut(&handle);
                parent.remove_child(*child_index);
                adjust_handles_for_delete(&mut new_to_delete, &handle);
                if parent.children().is_empty() {
                    new_to_delete.push(parent.handle());
                }
            }

            to_delete = new_to_delete;
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

/// When moving through a node, the cursor counts as inside the node
/// at one end, but not the other. This function determines that.
fn offset_is_inside_node(
    current_offset: usize,
    node_length: usize,
    direction: &Direction,
) -> bool {
    match direction {
        Direction::Forwards => current_offset < node_length,
        Direction::Backwards => current_offset > 0,
    }
}

/// Given a character, determine it's type
fn get_char_type(c: char) -> CharType {
    // in order to determine where a ctrl/opt + delete type operation finishes
    // we need to distinguish between whitespace (nb no newline characters), punctuation
    // and then everything else is treated as the same type
    if c.is_whitespace() {
        return CharType::Whitespace;
    }

    if c.is_ascii_punctuation() || c == '£' {
        // manually adding £ sign so it gets removed too
        return CharType::Punctuation;
    }

    return CharType::Other;
}

/// Given a node, return the type of character adjacent to the cursor offset position
/// bearing in mind the direction
fn get_char_type_from_node<S: UnicodeString>(
    node: &DomNode<S>,
    offset: usize,
    direction: &Direction,
) -> Option<CharType> {
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
            get_char_type_from_text_node(text_node, offset, direction)
        }
    }
}

// Separate function as we also call this directly
fn get_char_type_from_text_node<S: UnicodeString>(
    text_node: &TextNode<S>,
    offset: usize,
    direction: &Direction,
) -> Option<CharType> {
    let current_char = text_node
        .data()
        .chars()
        .nth(direction.get_index_from_cursor(offset));

    match current_char {
        Some(c) => Some(get_char_type(c)),
        None => None,
    }
}

/// Given a container search through it looking for text like nodes and return the first one
/// that we find (given the cursor position and direction)
fn recursively_search_container<'a, S: UnicodeString>(
    container: &'a ContainerNode<S>,
    direction: &'a Direction,
    cursor_position: usize,
    location: &DomLocation,
) -> Option<(DomHandle, usize, usize)> {
    // we need to keep track of a current offset for the case where we have a container
    // that contains multiple adjacent text nodes
    let mut current_container_offset: usize = 0;

    // loop through the children, if it's a container, recursively call this function
    // if it's a linebreak, return that node's details
    // if it's a text node, make sure we handle adjacent unmerged text nodes
    for child in container.children() {
        match child {
            DomNode::Container(node) => {
                recursively_search_container(
                    node,
                    direction,
                    cursor_position,
                    location,
                );
            }
            DomNode::LineBreak(_) => {
                return Some((
                    child.handle(),
                    location.position,
                    location.start_offset,
                ))
            }
            DomNode::Text(node) => {
                // want to ignore text nodes that are zero length and in the case
                // of adjacent text nodes return the correct one
                let node_length = node.data().len();
                let has_length = node_length != 0;

                let node_start = location.position + current_container_offset;
                let node_end = node_start + node_length;

                let node_offset = cursor_position - node_start;

                let is_inside_node = cursor_position < node_end;
                let is_correct_adjacent_node = match direction {
                    Direction::Forwards => node_offset == 0,
                    Direction::Backwards => node_offset == node_length,
                };

                if has_length && (is_inside_node || is_correct_adjacent_node) {
                    return Some((child.handle(), node_start, node_offset));
                }

                // to handle adjacent text nodes, increment the current offset in this container
                current_container_offset += node_length;
            }
        };
    }
    return None;
}

fn starts_with(subject: &DomHandle, object: &DomHandle) -> bool {
    // Can't start with something longer than you
    if subject.raw().len() < object.raw().len() {
        return false;
    }

    // If any path element doesn't match we don't start with this
    for (s, o) in subject.raw().iter().zip(object.raw().iter()) {
        if s != o {
            return false;
        }
    }

    // All elements match, so we do start with it
    true
}

fn adjust_handles_for_delete(
    handles: &mut Vec<DomHandle>,
    deleted: &DomHandle,
) {
    let mut indices_in_handles_to_delete = Vec::new();
    let mut handles_to_replace = Vec::new();

    let parent = deleted.parent_handle();
    for (i, handle) in handles.iter().enumerate() {
        if starts_with(handle, deleted) {
            // We are the deleted node (or a descendant of it)
            indices_in_handles_to_delete.push(i);
        } else if starts_with(handle, &parent) {
            // We are a sibling of the deleted node (or a descendant of one)

            // If we're after a deleted node, reduce our index
            let mut child_index = handle.raw()[parent.raw().len()];
            let deleted_index = *deleted.raw().last().unwrap();
            if child_index > deleted_index {
                child_index -= 1;
            }

            // Create a handle with the adjusted index (but missing anything
            // after the delete node's length).
            let mut new_handle = parent.child_handle(child_index);

            // Add back the rest of our original handle, unadjusted
            for h in &handle.raw()[deleted.raw().len()..] {
                new_handle = new_handle.child_handle(*h);
            }
            handles_to_replace.push((i, new_handle));
        }
    }

    for (i, new_handle) in handles_to_replace {
        handles[i] = new_handle;
    }

    indices_in_handles_to_delete.reverse();
    for i in indices_in_handles_to_delete {
        handles.remove(i);
    }
}

#[cfg(test)]
mod test {
    use crate::dom::DomHandle;

    use super::*;

    #[test]
    fn starts_with_works() {
        let h0123 = DomHandle::from_raw(vec![0, 1, 2, 3]);
        let h012 = DomHandle::from_raw(vec![0, 1, 2]);
        let h123 = DomHandle::from_raw(vec![1, 2, 3]);
        let h = DomHandle::from_raw(vec![]);

        assert!(starts_with(&h0123, &h012));
        assert!(!starts_with(&h012, &h0123));
        assert!(starts_with(&h012, &h012));
        assert!(starts_with(&h012, &h));
        assert!(!starts_with(&h123, &h012));
        assert!(!starts_with(&h012, &h123));
    }

    #[test]
    fn can_adjust_handles_when_removing_nodes() {
        let mut handles = vec![
            DomHandle::from_raw(vec![1, 2, 3]), // Ignored because before
            DomHandle::from_raw(vec![2, 3, 4, 5]), // Deleted because inside
            DomHandle::from_raw(vec![3, 4, 5]), // Adjusted because after
            DomHandle::from_raw(vec![3]),       // Adjusted because after
        ];

        let to_delete = DomHandle::from_raw(vec![2]);

        adjust_handles_for_delete(&mut handles, &to_delete);

        assert_eq!(*handles[0].raw(), vec![1, 2, 3]);
        assert_eq!(*handles[1].raw(), vec![2, 4, 5]);
        assert_eq!(*handles[2].raw(), vec![2]);
        assert_eq!(handles.len(), 3);
    }

    #[test]
    fn can_adjust_handles_when_removing_nested_nodes() {
        let mut handles = vec![
            DomHandle::from_raw(vec![0, 9, 1, 2, 3]),
            DomHandle::from_raw(vec![0, 9, 2, 3, 4, 5]),
            DomHandle::from_raw(vec![0, 9, 3, 4, 5]),
            DomHandle::from_raw(vec![0, 9, 3]),
        ];

        let to_delete = DomHandle::from_raw(vec![0, 9, 2]);

        adjust_handles_for_delete(&mut handles, &to_delete);

        assert_eq!(*handles[0].raw(), vec![0, 9, 1, 2, 3]);
        assert_eq!(*handles[1].raw(), vec![0, 9, 2, 4, 5]);
        assert_eq!(*handles[2].raw(), vec![0, 9, 2]);
        assert_eq!(handles.len(), 3);
    }
}
