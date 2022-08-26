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

use crate::dom::nodes::{ContainerNode, DomNode, TextNode};
use crate::dom::parser::parse;
use crate::dom::UnicodeString;
use crate::dom::{DomHandle, MultipleNodesRange, Range, SameNodeRange, ToHtml};
use crate::NodeJoiner;
use crate::{
    ActionResponse, ComposerState, ComposerUpdate, InlineFormatType, Location,
};

#[derive(Clone)]
pub struct ComposerModel<S>
where
    S: UnicodeString,
{
    pub state: ComposerState<S>,
    pub previous_states: Vec<ComposerState<S>>,
    pub next_states: Vec<ComposerState<S>>,
}

impl<'a, S> ComposerModel<S>
where
    S: UnicodeString,
{
    pub fn new() -> Self {
        Self {
            state: ComposerState::new(),
            previous_states: Vec::new(),
            next_states: Vec::new(),
        }
    }

    /**
     * Cursor is at end.
     */
    pub fn select(&mut self, start: Location, end: Location) {
        self.state.start = start;
        self.state.end = end;
    }

    /**
     * Return the start and end of the selection, ensuring the first number
     * returned is <= the second, and they are both 0<=n<=html.len().
     */
    fn safe_selection(&self) -> (usize, usize) {
        // TODO: Does not work with tags, and will probably be obselete when
        // we can look for ranges properly.
        let html = self.state.dom.to_html();

        let mut s: usize = self.state.start.into();
        let mut e: usize = self.state.end.into();
        s = s.clamp(0, html.len());
        e = e.clamp(0, html.len());
        if s > e {
            (e, s)
        } else {
            (s, e)
        }
    }

    /**
     * Replaces text in the current selection with new_text.
     */
    pub fn replace_text(&mut self, new_text: S) -> ComposerUpdate<S> {
        // TODO: escape any HTML?
        let (s, e) = self.safe_selection();
        self.replace_text_in(new_text, s, e)
    }

    /**
     * Replaces text in the an arbitrary start..end range with new_text.
     */
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
                self.state
                    .dom
                    .append_child(DomNode::Text(TextNode::from(new_text)));

                start = 0;
            }
        }

        self.state.start = Location::from(start + len);
        self.state.end = self.state.start;

        // TODO: for now, we replace every time, to check ourselves, but
        // at least some of the time we should not
        self.create_update_replace_all()
    }

    pub fn backspace(&mut self) -> ComposerUpdate<S> {
        if self.state.start == self.state.end {
            // Go back 1 from the current location
            self.state.start -= 1;
        }

        self.replace_text(S::new())
    }

    /**
     * Deletes text in an arbitrary start..end range.
     */
    pub fn delete_in(&mut self, start: usize, end: usize) -> ComposerUpdate<S> {
        self.state.end = Location::from(start);
        self.replace_text_in(S::new(), start, end)
    }

    /**
     * Deletes the character after the current cursor position.
     */
    pub fn delete(&mut self) -> ComposerUpdate<S> {
        if self.state.start == self.state.end {
            // Go forward 1 from the current location
            self.state.end += 1;
        }

        self.replace_text(S::new())
    }

    pub fn action_response(
        &mut self,
        action_id: String,
        response: ActionResponse,
    ) -> ComposerUpdate<S> {
        drop(action_id);
        drop(response);
        ComposerUpdate::keep()
    }

    pub fn get_selection(&self) -> (Location, Location) {
        (self.state.start, self.state.end)
    }

    pub fn format(&mut self, format: InlineFormatType) -> ComposerUpdate<S> {
        // Store current Dom
        self.push_state_to_history();
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        match range {
            Range::SameNode(range) => {
                self.format_same_node(range, format);
                // TODO: for now, we replace every time, to check ourselves, but
                // at least some of the time we should not
                return self.create_update_replace_all();
            }

            Range::NoNode => {
                self.state.dom.append_child(DomNode::new_formatting(
                    S::from_str(format.tag()),
                    vec![DomNode::Text(TextNode::from(S::from_str("")))],
                ));
                return ComposerUpdate::keep();
            }

            _ => panic!("Can't format in complex object models yet"),
        }
    }

    pub fn create_ordered_list(&mut self) -> ComposerUpdate<S> {
        self.create_list(true)
    }

    pub fn create_unordered_list(&mut self) -> ComposerUpdate<S> {
        self.create_list(false)
    }

    pub fn get_html(&self) -> S {
        self.state.dom.to_html()
    }

    pub fn undo(&mut self) -> ComposerUpdate<S> {
        if let Some(prev) = self.previous_states.pop() {
            self.next_states.push(self.state.clone());
            self.state = prev;
            self.create_update_replace_all()
        } else {
            ComposerUpdate::keep()
        }
    }

    pub fn redo(&mut self) -> ComposerUpdate<S> {
        if let Some(next) = self.next_states.pop() {
            self.previous_states.push(self.state.clone());
            self.state = next;
            self.create_update_replace_all()
        } else {
            ComposerUpdate::keep()
        }
    }

    pub fn get_current_state(&self) -> &ComposerState<S> {
        &self.state
    }

    // Internal functions
    fn create_update_replace_all(&self) -> ComposerUpdate<S> {
        ComposerUpdate::replace_all(
            self.state.dom.to_html(),
            self.state.start,
            self.state.end,
        )
    }

    fn create_list(&mut self, ordered: bool) -> ComposerUpdate<S> {
        // Store current Dom
        self.push_state_to_history();
        let list_tag = if ordered { "ol" } else { "ul" };
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        match range {
            Range::SameNode(range) => {
                let node =
                    self.state.dom.lookup_node(range.node_handle.clone());
                if let DomNode::Text(t) = node {
                    let text = t.data();
                    let list_node = DomNode::new_list(
                        S::from_str(list_tag),
                        vec![DomNode::Container(ContainerNode::new_list_item(
                            S::from_str("li"),
                            vec![DomNode::Text(TextNode::from(text.clone()))],
                        ))],
                    );
                    self.state.dom.replace(range.node_handle, vec![list_node]);
                    return self.create_update_replace_all();
                } else {
                    panic!("Can't create a list from a non-text node")
                }
            }

            Range::NoNode => {
                self.state.dom.append_child(DomNode::new_list(
                    S::from_str(list_tag),
                    vec![DomNode::Container(ContainerNode::new_list_item(
                        S::from_str("li"),
                        vec![DomNode::Text(TextNode::from(S::from_str("")))],
                    ))],
                ));
                return self.create_update_replace_all();
            }

            _ => {
                panic!("Can't create ordered list in complex object models yet")
            }
        }
    }

    fn replace_same_node(&mut self, range: SameNodeRange, new_text: S) {
        // TODO: remove SameNode and NoNode?
        let node = self.state.dom.lookup_node_mut(range.node_handle);
        if let DomNode::Text(ref mut t) = node {
            let text = t.data();
            let mut n = slice_to(text, ..range.start_offset);
            n.push_string(&new_text);
            n.push_string(&slice_from(&text, range.end_offset..));
            t.set_data(n);
        } else {
            panic!("Can't deal with ranges containing non-text nodes (yet?)")
        }
    }

    fn replace_multiple_nodes(
        &mut self,
        range: MultipleNodesRange,
        new_text: S,
    ) {
        let len = new_text.len();
        let node_joiner = NodeJoiner::from_range(&self.state.dom, &range);

        let to_delete = self.replace_in_text_nodes(range, new_text);
        self.delete_nodes(to_delete);

        let pos: usize = self.state.start.into();
        node_joiner.join_nodes(&mut self.state.dom, pos + len);
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
            let mut node =
                self.state.dom.lookup_node_mut(loc.node_handle.clone());
            match &mut node {
                DomNode::Container(_) => {
                    // Nothing to do for container nodes
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

    fn delete_nodes(&mut self, mut to_delete: Vec<DomHandle>) {
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
                let mut parent =
                    self.state.dom.lookup_node_mut(parent_handle.clone());
                match &mut parent {
                    DomNode::Container(parent) => {
                        parent.remove_child(*child_index);
                        adjust_handles_for_delete(&mut new_to_delete, &handle);
                        if parent.children().is_empty() {
                            new_to_delete.push(parent_handle);
                        }
                    }
                    DomNode::Text(_) => {
                        panic!("Parent must be a container!");
                    }
                }
            }

            to_delete = new_to_delete;
        }
    }

    fn format_same_node(
        &mut self,
        range: SameNodeRange,
        format: InlineFormatType,
    ) {
        let node = self.state.dom.lookup_node(range.node_handle.clone());
        if let DomNode::Text(t) = node {
            let text = t.data();
            // TODO: can we be globally smart about not leaving empty text nodes ?
            let before = slice_to(text, ..range.start_offset);
            let during = slice(text, range.start_offset..range.end_offset);
            let after = slice_from(text, range.end_offset..);
            let new_nodes = vec![
                DomNode::Text(TextNode::from(before)),
                DomNode::new_formatting(
                    S::from_str(format.tag()),
                    vec![DomNode::Text(TextNode::from(during))],
                ),
                DomNode::Text(TextNode::from(after)),
            ];
            self.state.dom.replace(range.node_handle, new_nodes);
        } else {
            panic!("Trying to bold a non-text node")
        }
    }

    fn push_state_to_history(&mut self) {
        // Clear future events as they're no longer valid
        self.next_states.clear();
        // Store a copy of the current state in the previous_states
        self.previous_states.push(self.state.clone());
    }

    pub fn enter(&mut self) -> ComposerUpdate<S> {
        // Store current Dom
        self.push_state_to_history();
        let (s, e) = self.safe_selection();
        let range = self.state.dom.find_range(s, e);
        match range {
            Range::SameNode(range) => {
                let parent_list_item_handle = self
                    .state
                    .dom
                    .find_parent_list_item(range.node_handle.clone());
                if let Some(parent_handle) = parent_list_item_handle {
                    let parent_node =
                        self.state.dom.lookup_node(parent_handle.clone());
                    let list_node_handle = parent_node.handle().parent_handle();
                    if let DomNode::Container(parent) = parent_node {
                        if parent.is_empty_list_item() {
                            self.remove_list_item(
                                list_node_handle,
                                e,
                                parent_handle.index_in_parent(),
                            );
                        } else {
                            self.add_list_item(list_node_handle, e);
                        }
                        self.create_update_replace_all()
                    } else {
                        panic!("No list item found")
                    }
                } else {
                    self.replace_text(S::from_str("\n"))
                }
            }
            _ => self.replace_text(S::from_str("\n")),
        }
    }

    pub fn replace_all_html(&mut self, html: &S) -> ComposerUpdate<S> {
        let dom = parse(&html.to_utf8());

        match dom {
            Ok(dom) => {
                self.state.dom = dom;
                self.create_update_replace_all()
            }
            Err(e) => {
                // TODO: log error
                self.state.dom = e.dom;
                self.create_update_replace_all()
            }
        }
    }

    pub fn set_link(&mut self, link: S) -> ComposerUpdate<S> {
        let (s, e) = self.safe_selection();
        // Can't add a link to an empty selection
        if s == e {
            return ComposerUpdate::keep();
        }
        // Store current Dom
        self.push_state_to_history();

        let range = self.state.dom.find_range(s, e);
        match range {
            Range::SameNode(range) => {
                self.set_link_same_node(range, link);
                // TODO: for now, we replace every time, to check ourselves, but
                // at least some of the time we should not
                return self.create_update_replace_all();
            }

            Range::NoNode => {
                panic!("Can't add link to empty range");
            }

            _ => panic!("Can't add link in complex object models yet"),
        }
    }

    fn set_link_same_node(&mut self, range: SameNodeRange, link: S) {
        // TODO: set link should be able to wrap container nodes, unlike formatting
        let node = self.state.dom.lookup_node(range.node_handle.clone());
        if let DomNode::Text(t) = node {
            let text = t.data();
            // TODO: can we be globally smart about not leaving empty text nodes ?
            let before = slice_to(text, ..range.start_offset);
            let during = slice(text, range.start_offset..range.end_offset);
            let after = slice_from(text, range.end_offset..);
            let new_nodes = vec![
                DomNode::Text(TextNode::from(before)),
                DomNode::new_link(
                    link,
                    vec![DomNode::Text(TextNode::from(during))],
                ),
                DomNode::Text(TextNode::from(after)),
            ];
            self.state.dom.replace(range.node_handle, new_nodes);
        } else {
            panic!("Trying to bold a non-text node")
        }
    }

    fn add_list_item(&mut self, handle: DomHandle, location: usize) {
        let list_node = self.state.dom.lookup_node_mut(handle);
        if let DomNode::Container(list) = list_node {
            list.append_child(DomNode::new_list_item(
                S::from_str("li"),
                vec![DomNode::Text(TextNode::from(S::from_str("\u{200B}")))],
            ));
            self.state.start = Location::from(location + 1);
            self.state.end = Location::from(location + 1);
        } else {
            panic!("Handle doesn't match a list container node")
        }
    }

    fn remove_list_item(
        &mut self,
        handle: DomHandle,
        location: usize,
        list_item_index: usize,
    ) {
        let list_node = self.state.dom.lookup_node_mut(handle.clone());
        if let DomNode::Container(list) = list_node {
            if list.children().len() == 1 {
                let parent_handle = handle.parent_handle();
                let parent_node = self.state.dom.lookup_node_mut(parent_handle);
                if let DomNode::Container(parent) = parent_node {
                    parent.remove_child(handle.index_in_parent());
                    if parent.children().len() == 0 {
                        parent.append_child(DomNode::Text(TextNode::from(
                            S::from_str(""),
                        )));
                    }
                    self.state.start = Location::from(location);
                    self.state.end = Location::from(location);
                } else {
                    // TODO: handle list items outside of lists
                    panic!("List has no parent container")
                }
            } else {
                list.remove_child(list_item_index);
                let parent_handle = handle.parent_handle();
                let parent_node = self.state.dom.lookup_node_mut(parent_handle);
                if let DomNode::Container(parent) = parent_node {
                    // TODO: should probably append a paragraph instead
                    parent.append_child(DomNode::Text(TextNode::from(
                        S::from_str("\u{200B}"),
                    )));
                    self.state.start = Location::from(location);
                    self.state.end = Location::from(location);
                } else {
                    panic!("List has no parent container")
                }
            }
        }
    }
}

fn slice_to<S>(s: &S, range: std::ops::RangeTo<usize>) -> S
where
    S: UnicodeString,
{
    slice(s, 0..range.end)
}

fn slice_from<S>(s: &S, range: std::ops::RangeFrom<usize>) -> S
where
    S: UnicodeString,
{
    slice(s, range.start..s.len())
}

/// Panics when given start or end not on boundaries of a code point
/// TODO: don't panic but do something sensible in that case
fn slice<S>(s: &S, range: std::ops::Range<usize>) -> S
where
    S: UnicodeString,
{
    S::from_vec(s.as_slice()[range].to_vec()).expect("Invalid slice!")
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
    use widestring::Utf16String;

    use super::*;
    use crate::tests::testutils_composer_model::cm;

    use crate::dom::DomHandle;

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
