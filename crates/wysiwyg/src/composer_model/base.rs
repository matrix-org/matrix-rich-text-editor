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

use crate::composer_state::ComposerState;
use crate::dom::parser::parse;
use crate::dom::{DomHandle, UnicodeString};
use crate::{ComposerAction, ComposerUpdate, Location, ToHtml, ToTree};
use std::collections::HashSet;

#[derive(Clone)]
pub struct ComposerModel<S>
where
    S: UnicodeString,
{
    pub state: ComposerState<S>,
    pub previous_states: Vec<ComposerState<S>>,
    pub next_states: Vec<ComposerState<S>>,
    pub reversed_actions: HashSet<ComposerAction>,
    pub disabled_actions: HashSet<ComposerAction>,
}

impl<S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn new() -> Self {
        Self {
            state: ComposerState::new(),
            previous_states: Vec::new(),
            next_states: Vec::new(),
            reversed_actions: HashSet::new(),
            disabled_actions: HashSet::new(),
        }
    }

    pub fn from_state(state: ComposerState<S>) -> Self {
        Self {
            state: state,
            previous_states: Vec::new(),
            next_states: Vec::new(),
            reversed_actions: HashSet::new(),
            disabled_actions: HashSet::new(),
        }
    }

    /// Create a UTF-16 model from an HTML string, or panic if HTML parsing
    /// fails.
    pub fn from_html(
        html: &str,
        start_codeunit: usize,
        end_codeunit: usize,
    ) -> Self {
        let mut model = Self {
            state: ComposerState {
                dom: parse(html).expect("HTML parsing failed"),
                start: Location::from(start_codeunit),
                end: Location::from(end_codeunit),
                toggled_format_types: Vec::new(),
            },
            previous_states: Vec::new(),
            next_states: Vec::new(),
            reversed_actions: HashSet::new(),
            disabled_actions: HashSet::new(),
        };
        model.compute_menu_state();
        model
    }

    /// Replace the entire content of the model with given HTML string.
    /// This will remove all previous and next states, effectively disabling
    /// undo and redo until further updates.
    pub fn replace_all_html(&mut self, html: &S) -> ComposerUpdate<S> {
        let dom = parse(&html.to_string());

        match dom {
            Ok(dom) => {
                self.state.dom = dom;
                self.previous_states.clear();
                self.next_states.clear();
                self.create_update_replace_all()
            }
            Err(e) => {
                // We should log here - internal task PSU-741
                self.state.dom = e.dom;
                self.previous_states.clear();
                self.next_states.clear();
                self.create_update_replace_all()
            }
        }
    }

    pub(crate) fn create_update_replace_all(&mut self) -> ComposerUpdate<S> {
        ComposerUpdate::replace_all(
            self.state.dom.to_html(),
            self.state.start,
            self.state.end,
            self.compute_menu_state(),
        )
    }

    pub fn get_selection(&self) -> (Location, Location) {
        (self.state.start, self.state.end)
    }

    pub fn get_html(&self) -> S {
        self.state.dom.to_html()
    }

    pub fn get_current_state(&self) -> &ComposerState<S> {
        &self.state
    }

    pub fn to_tree(&self) -> S {
        self.state.dom.to_tree()
    }
}

pub(crate) fn starts_with(subject: &DomHandle, object: &DomHandle) -> bool {
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

pub(crate) fn adjust_handles_for_delete(
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
    use widestring::Utf16String;

    use crate::dom::DomHandle;
    use crate::tests::testutils_composer_model::cm;

    use super::*;

    // Most tests for ComposerModel are inside the tests/ modules

    #[test]
    fn completely_replacing_html_works() {
        let mut model = cm("{hello}| world");
        model.replace_all_html(&Utf16String::from_str("foo <b>bar</b>"));
        assert_eq!(model.state.dom.to_string(), "foo <b>bar</b>");
    }

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
