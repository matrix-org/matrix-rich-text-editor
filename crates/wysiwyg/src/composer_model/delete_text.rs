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

use crate::dom::nodes::{DomNode, TextNode};
use crate::dom::unicode_string::UnicodeStrExt;
use crate::dom::{DomHandle, DomLocation, Range};
use crate::{ComposerModel, ComposerUpdate, Location, UnicodeString};

// categories of character
#[derive(PartialEq, Debug)]
enum Cat {
    Whitespace,
    Newline,
    Punctuation,
    Other,
    None,
}

#[derive(PartialEq, Debug)]
enum Dir {
    Forwards,
    Backwards,
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

        // next actions depend on start type
        let start_type = self.get_char_type_at(e);
        match start_type {
            Cat::Whitespace => {
                let (ws_delete_index, stopped_at_newline) =
                    self.get_end_index_of_run(e, &Dir::Forwards);
                if stopped_at_newline {
                    // +2 to account for the fact we want to remove the newline
                    return self.delete_in(s, ws_delete_index + 2);
                } else {
                    self.delete_in(s, ws_delete_index + 1);
                    let (_s, _e) = self.safe_selection();
                    let (non_ws_delete_index, _) =
                        self.get_end_index_of_run(_e + 1, &Dir::Forwards);
                    return self.delete_in(_s, non_ws_delete_index + 1);
                }
            }
            Cat::Newline => {
                return self.delete_in(s, e + 1);
            }
            Cat::Punctuation => {
                let (delete_index, _) =
                    self.get_end_index_of_run(e + 1, &Dir::Forwards);
                return self.delete_in(s, delete_index + 1);
            }
            Cat::Other => {
                let (delete_index, _) =
                    self.get_end_index_of_run(e + 1, &Dir::Forwards);
                return self.delete_in(s, delete_index + 1);
            }
            Cat::None => ComposerUpdate::keep(),
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

        // next actions depend on start type
        let start_type = self.get_char_type_at(e - 1);
        match start_type {
            Cat::Whitespace => {
                let (ws_delete_index, stopped_at_newline) =
                    self.get_end_index_of_run(e - 1, &Dir::Backwards);
                if stopped_at_newline {
                    // -1 to account for the fact we want to remove the newline
                    return self.delete_in(ws_delete_index - 1, e);
                } else {
                    self.delete_in(ws_delete_index, e);
                    let (_s, _e) = self.safe_selection();
                    let (non_ws_delete_index, _) =
                        self.get_end_index_of_run(_e - 1, &Dir::Backwards);
                    return self.delete_in(non_ws_delete_index, _e);
                }
            }
            Cat::Newline => {
                return self.delete_in(s - 1, e);
            }
            Cat::Punctuation => {
                let (delete_index, _) =
                    self.get_end_index_of_run(e - 1, &Dir::Backwards);
                return self.delete_in(delete_index, e);
            }
            Cat::Other => {
                let (delete_index, _) =
                    self.get_end_index_of_run(e - 1, &Dir::Backwards);
                return self.delete_in(delete_index, e);
            }
            Cat::None => ComposerUpdate::keep(),
        }
    }

    // types defined in the Cat struct
    fn get_char_type_at(&self, index: usize) -> Cat {
        let content = self.state.dom.to_string();

        match content.chars().nth(index) {
            Some(c) => {
                // handle newlines separately, otherwise they'll get classed as white space,
                // do we want to also have \r in here?
                if c == '\n' {
                    return Cat::Newline;
                }

                if c.is_whitespace() {
                    return Cat::Whitespace;
                } else if c.is_ascii_punctuation() || c == '£' {
                    // is_ascii_punctuation doesn't include £, do we want to manually add this?
                    return Cat::Punctuation;
                } else {
                    return Cat::Other;
                }
            }
            None => Cat::None,
        }
    }

    // figure out where the run ends and also if we're returning due to a
    // newline (true) or a change in character type (false)
    fn get_end_index_of_run(
        &self,
        start: usize,
        direction: &Dir,
    ) -> (usize, bool) {
        let start_type = self.get_char_type_at(start);
        let mut current_index = start.clone();
        let mut current_type = self.get_char_type_at(current_index);
        let mut stopped_at_newline = start_type.eq(&Cat::Newline);
        let mut would_hit_end = false;

        fn increment(index: usize, dir: &Dir) -> usize {
            match dir {
                Dir::Backwards => index - 1,
                Dir::Forwards => index + 1,
            }
        }
        fn decrement(index: usize, dir: &Dir) -> usize {
            match dir {
                Dir::Backwards => index + 1,
                Dir::Forwards => index - 1,
            }
        }

        fn check_condition(
            index: usize,
            max: usize,
            start_type: &Cat,
            current_type: &Cat,
            dir: &Dir,
            stopped_at_newline: bool,
        ) -> bool {
            let base_condition =
                current_type.eq(start_type) && !stopped_at_newline;
            match dir {
                Dir::Backwards => base_condition && index > 0,
                Dir::Forwards => base_condition && index < max,
            }
        }

        while check_condition(
            current_index,
            self.state.dom.text_len(),
            &start_type,
            &current_type,
            direction,
            stopped_at_newline,
        ) {
            current_index = increment(current_index, direction);
            current_type = self.get_char_type_at(current_index);
            if current_type.eq(&start_type)
                && (current_index == 0
                    || current_index == self.state.dom.text_len())
            {
                would_hit_end = true;
            }
            if current_type.eq(&Cat::Newline) {
                stopped_at_newline = true;
            }
        }

        // if it would have hit the end of the string, return that index, otherwise
        // return the index of the end of the run
        match would_hit_end {
            true => (current_index, stopped_at_newline),
            false => (decrement(current_index, direction), stopped_at_newline),
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
