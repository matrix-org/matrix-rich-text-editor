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

use crate::dom::nodes::{ContainerNode, DomNode, TextNode};
use crate::dom::unicode_string::{UnicodeStr, UnicodeStrExt};
use crate::dom::{DomHandle, DomLocation, Range};
use crate::{ComposerModel, ComposerUpdate, Location, UnicodeString};

// categories of character
#[derive(PartialEq, Debug)]
enum CharType {
    Whitespace,
    Newline,
    Punctuation,
    Other,
    None,
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

    /// Recursively searches the dom to find the next char type, only ever
    /// called with a range where start == end
    fn get_char_type_at_cursor_position(
        &self,
        range: &Range,
        direction: &Direction,
    ) -> CharType {
        let first_leaf = range.locations.iter().find(|loc| loc.is_leaf);
        // issue here is that the assumption that we want to use the first leaf doesn't hold in the case
        // where we have a cursor on the boundary of two leaves and we're going forwards. in that case,
        // increment to go to the second leaf
        if let Some(leaf) = first_leaf {
            let mut my_dom_node = self.state.dom.lookup_node(&leaf.node_handle);

            let increment_leaf = leaf.start_offset == leaf.length
                && direction.eq(&Direction::Forwards);
            if increment_leaf {
                println!("should increment leaf");
                let next_sibling = &leaf.node_handle.next_sibling();
                let next_node = self.state.dom.lookup_node(next_sibling);
                println!("leaf stuff - {:?}", next_node);
                my_dom_node = self.state.dom.lookup_node(next_sibling)
            }

            println!("dom node: {:?}", my_dom_node);

            match my_dom_node {
                DomNode::Container(node) => {
                    return CharType::Other;
                }
                DomNode::Text(node) => {
                    if node.data().len() == 0 {
                        // I have no idea why a line break in this test case "|<br/> abc"
                        // ends up hitting here... surely it should be a line break type,
                        // not an empty node? Is this an issue somewhere else?
                        // "abc <br />|" gets identified properly. Weird.
                        println!("have hit empty node, is this a linebreak?");
                        return CharType::Newline;
                    }
                    let content = node.data();
                    let leaf_cursor = leaf.start_offset;
                    let n = direction.get_index_from_cursor(leaf_cursor);

                    let nth_char = content.chars().nth(n);
                    return match nth_char {
                        Some(c) => {
                            if c.is_whitespace() {
                                return CharType::Whitespace;
                            } else if c.is_ascii_punctuation() || c == '£' {
                                // is_ascii_punctuation doesn't include £, do we want to manually add this?
                                return CharType::Punctuation;
                            } else {
                                return CharType::Other;
                            }
                        }
                        None => {
                            println!("no char!");
                            CharType::None
                        }
                    };
                }
                DomNode::LineBreak(node) => {
                    return CharType::Newline;
                }
            };
        } else {
            println!("no leaf!");
            return CharType::None;
        };
    }

    /// Implements the ctrl/opt + delete/backspace functionality
    fn remove_word_in_direction(
        &mut self,
        direction: &Direction,
    ) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);

        // if we have a selection, only remove the selection
        if range.start() != range.end() {
            return self.delete_in(s, e);
        }

        // if we are trying to move outside the dom, do nothing
        if (direction.eq(&Direction::Forwards)
            && range.start() == self.state.dom.text_len())
            || (direction.eq(&Direction::Backwards) && range.start() == 0)
        {
            return ComposerUpdate::keep();
        }

        // from here on, start == end, so let's make a cursor position called cursor
        let cursor = range.start();
        println!("<<< LOGGING >>>");
        // next actions depend on start type
        let start_type =
            self.get_char_type_at_cursor_position(&range, direction);

        println!("CONTENT    -- {:?}", self.state.dom.to_string());
        println!("CURSOR AT  -- {:?}", cursor);
        println!("START TYPE -- {:?}", start_type);

        return match start_type {
            CharType::Whitespace => {
                match self.get_end_index_of_run(&range, direction, &start_type)
                {
                    None => return ComposerUpdate::keep(),
                    Some((ws_delete_cursor, stopped_at_newline)) => {
                        match stopped_at_newline {
                            // +2 to account for the fact we want to remove the newline
                            true => match direction {
                                Direction::Forwards => self.delete_in(
                                    cursor,
                                    direction.increment(ws_delete_cursor),
                                ),
                                Direction::Backwards => self.delete_in(
                                    direction.increment(ws_delete_cursor),
                                    cursor,
                                ),
                            },
                            false => {
                                match direction {
                                    Direction::Forwards => {
                                        self.delete_in(cursor, ws_delete_cursor)
                                    }
                                    Direction::Backwards => {
                                        self.delete_in(ws_delete_cursor, cursor)
                                    }
                                };
                                let (_s, _e) = self.safe_selection();
                                let _range = self.state.dom.find_range(_s, _e);
                                let _cursor = _range.start();
                                let _start_type = self
                                    .get_char_type_at_cursor_position(
                                        &_range, direction,
                                    );
                                println!(
                                    "second cursor: {}, second type - {:?}",
                                    _cursor, _start_type
                                );

                                match self.get_end_index_of_run(
                                    &_range,
                                    direction,
                                    &_start_type,
                                ) {
                                    None => return ComposerUpdate::keep(),
                                    Some((
                                        next_delete_cursor,
                                        stopped_at_newline,
                                    )) => match direction {
                                        Direction::Forwards => self.delete_in(
                                            _cursor,
                                            next_delete_cursor,
                                        ),

                                        Direction::Backwards => self.delete_in(
                                            next_delete_cursor,
                                            _cursor,
                                        ),
                                    },
                                }
                            }
                        }
                    }
                }
            }
            CharType::Newline | CharType::Punctuation | CharType::Other => {
                match self.get_end_index_of_run(&range, direction, &start_type)
                {
                    Some((delete_cursor, _)) => match direction {
                        Direction::Forwards => {
                            self.delete_in(cursor, delete_cursor)
                        }
                        Direction::Backwards => {
                            self.delete_in(delete_cursor, cursor)
                        }
                    },
                    None => ComposerUpdate::keep(),
                }
            }
            CharType::None => ComposerUpdate::keep(),
        };
    }

    /// Remove a single word when user does ctrl/cmd + delete
    pub fn delete_word(&mut self) -> ComposerUpdate<S> {
        self.remove_word_in_direction(&Direction::Forwards)
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
        self.remove_word_in_direction(&Direction::Backwards)
    }

    // I don't think we need to do it by index, lets just pass the char in
    // and this method can probably then become a util later on
    fn get_char_type(&self, char: Option<char>) -> CharType {
        if let Some(c) = char {
            if c.is_whitespace() {
                return CharType::Whitespace;
            } else if c.is_ascii_punctuation() || c == '£' {
                // is_ascii_punctuation doesn't include £, do we want to manually add this?
                return CharType::Punctuation;
            } else {
                return CharType::Other;
            }
        } else {
            CharType::None
        }
    }

    /// can become a util
    fn recursively_search_container<'a>(
        &'a self,
        container: &'a ContainerNode<S>,
        direction: &Direction,
    ) -> Option<&DomNode<S>> {
        for child in container.children().iter() {
            match child {
                DomNode::Container(node) => {
                    return self.recursively_search_container(node, direction);
                }
                DomNode::LineBreak(_) | DomNode::Text(_) => return Some(child),
            };
        }
        return None;
    }

    fn get_next_text_like_node_from_range(
        &self,
        range: &Range,
        direction: &Direction,
    ) -> Option<&DomNode<S>> {
        // this needs to be able to deal with the case where we're between containers
        let start_location = match direction {
            Direction::Forwards => range.locations.last(),
            Direction::Backwards => range.locations.first(),
        };
        match start_location {
            Some(loc) => {
                // to do move nodes when we're at the end
                if loc.start_offset == loc.length {}
                let dom_node = self.state.dom.lookup_node(&loc.node_handle);

                return match dom_node {
                    DomNode::Container(node) => {
                        self.recursively_search_container(node, direction)
                    }
                    DomNode::Text(_) => Some(dom_node),
                    DomNode::LineBreak(_) => Some(dom_node),
                };
            }
            None => None::<&DomNode<S>>,
        };
        return None;
    }

    // figure out where the run ends and also if we're returning due to a
    // newline (true) or a change in character type (false)

    // TODO make this take a type argument too, so that we can pass in the start type
    // or maybe we pass this an index and a type and it sorts the range out? then
    // we can just keep changing the index each time to move it around
    fn get_end_index_of_run(
        &self,
        range: &Range,
        direction: &Direction,
        start_type: &CharType,
    ) -> Option<(usize, bool)> {
        println!("---<<<--->>>---");
        println!("call get end of index run");
        println!("with start type of {:?}", start_type);
        // TODO need to give this some base cases (assuming it's passed a range)
        // so that we can go onto the recursive call phase, to keep going until either we
        // - hit the beginning or end of the dom X
        // - hit a br tag
        // - there's a change in character type
        // nb possible these will change with lists

        // base case
        let cursor = range.start();
        if cursor == self.state.dom.text_len()
            && direction.eq(&Direction::Forwards)
        {
            return Some((cursor, false));
        }
        if cursor == 0 && direction.eq(&Direction::Backwards) {
            return Some((cursor, false));
        }

        let next_text_node =
            self.get_next_text_like_node_from_range(range, direction);
        match next_text_node {
            Some(text_node) => {
                println!("first text node is: {:?}", text_node)
            }
            None => {
                println!("no textnodes")
            }
        };

        let first_leaf = range.locations.iter().find(|loc| loc.is_leaf);
        if let Some(leaf) = first_leaf {
            let mut my_dom_node = self.state.dom.lookup_node(&leaf.node_handle);
            let increment_leaf = leaf.start_offset == leaf.length
                && direction.eq(&Direction::Forwards);
            if increment_leaf {
                println!("should increment leaf");
                let next_sibling = &leaf.node_handle.next_sibling();
                let next_node = self.state.dom.lookup_node(next_sibling);
                println!("leaf stuff - {:?}", next_node);
                my_dom_node = self.state.dom.lookup_node(next_sibling);
            }
            match my_dom_node {
                DomNode::Container(node) => {
                    // need to probably sort this case out for lists,
                    let node =
                        node.children().iter().find(|node| node.is_text_node());
                    match node {
                        Some(node) => {
                            match node {
                                DomNode::Container(_) => todo!(),
                                DomNode::Text(_node) => {
                                    println!("node data - {:?}", _node.data());
                                }
                                DomNode::LineBreak(_) => todo!(),
                            };
                        }
                        None => todo!(),
                    };

                    panic!("we hit a container, perhaps handle this like a text node");
                }
                DomNode::Text(node) => {
                    let content = node.data();
                    if node.data().len() == 0 {
                        // panic!(
                        //     "we hit a zero length text node in test string: {}",
                        //     self.state.dom.to_string()
                        // );
                        // I have no idea why a line break in this test case "|<br/> abc"
                        // ends up hitting here... surely it should be a line break type,
                        // not an empty node? Is this an issue somewhere else?
                        // "abc <br />|" gets identified properly. Weird.
                        println!("have hit empty node, is this a linebreak?");
                        if start_type.eq(&CharType::Newline) {
                            return Some((direction.increment(cursor), true));
                        } else {
                            return Some((cursor, true));
                        };
                    }
                    // let start_index = match direction {
                    //     Direction::Forwards => leaf.start_offset,
                    //     Direction::Backwards => leaf.start_offset - 1,
                    // };
                    let leaf_cursor = leaf.start_offset;

                    // don't want to use the below, will get passed in the real start type
                    // as we may have traversed many leaves to get here
                    // let start_char = content.chars().nth(start_index);
                    // let start_type = self.get_char_type(start_char);
                    let mut current_leaf_cursor = leaf_cursor.clone();
                    let mut current_char = content.chars().nth(
                        direction.get_index_from_cursor(current_leaf_cursor),
                    );
                    let mut current_type = self.get_char_type(current_char);

                    let mut have_hit_leaf_end = false; // this will get easier using cursors and maybe can go
                    let mut remove_count: usize = 0; // always positive (just like me)

                    fn check_condition(
                        start_type: &CharType,
                        current_type: &CharType,
                        current_leaf_cursor: &usize,
                        leaf_length: &usize,
                        direction: &Direction,
                    ) -> bool {
                        let base_condition = current_type.eq(start_type);
                        return match direction {
                            Direction::Forwards => {
                                base_condition
                                    && current_leaf_cursor < leaf_length
                            }
                            Direction::Backwards => {
                                base_condition && current_leaf_cursor > &0
                            }
                        };
                    }

                    // TODO sort out the logic in here - we need to make sure we can go
                    // all the way up to the edge of the leaf without overflow in the index
                    // (because if we have a 0 cursor going backwards it will try to get at the -1th char)
                    while check_condition(
                        &start_type,
                        &current_type,
                        &current_leaf_cursor,
                        &leaf.length,
                        direction,
                    ) {
                        current_leaf_cursor =
                            direction.increment(current_leaf_cursor);
                        remove_count += 1; // as above

                        if current_leaf_cursor == 0
                            || current_leaf_cursor == leaf.length
                        {
                            have_hit_leaf_end = true;
                            break;
                        }

                        current_char =
                            content
                                .chars()
                                .nth(direction.get_index_from_cursor(
                                    current_leaf_cursor,
                                ));
                        current_type = self.get_char_type(current_char);
                    }

                    let delete_cursor = match direction {
                        Direction::Forwards => cursor + remove_count,
                        Direction::Backwards => cursor - remove_count,
                    };

                    // check for if we've hit the end of the dom
                    if delete_cursor == 0 && direction.eq(&Direction::Backwards)
                    {
                        return Some((delete_cursor, false));
                    }
                    if delete_cursor == self.state.dom.text_len()
                        && direction.eq(&Direction::Forwards)
                    {
                        return Some((delete_cursor, false));
                    }

                    if have_hit_leaf_end {
                        // make a new range...
                        let next_range = self
                            .state
                            .dom
                            .find_range(delete_cursor, delete_cursor);
                        // ...then make the recursive call
                        return self.get_end_index_of_run(
                            &next_range,
                            direction,
                            &start_type,
                        );
                    }

                    return Some((delete_cursor, false));
                }
                DomNode::LineBreak(node) => {
                    // increment if we started at a newline and we're at one of those dom nodes
                    // so that we remove those nodes
                    if start_type.eq(&CharType::Newline) {
                        return Some((direction.increment(cursor), true));
                    } else {
                        return Some((cursor, true));
                    };
                }
            };
        } else {
            panic!("shouldn't hit this");
        };
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
