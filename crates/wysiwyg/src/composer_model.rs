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
use crate::dom::{
    Dom, DomHandle, MultipleNodesRange, Range, SameNodeRange, ToHtml,
};
use crate::{
    ActionResponse, ComposerState, ComposerUpdate, InlineFormatType, Location,
};

#[derive(Clone)]
pub struct ComposerModel<C>
where
    C: Clone,
{
    pub state: ComposerState<C>,
    pub previous_states: Vec<ComposerState<C>>,
    pub next_states: Vec<ComposerState<C>>,
}

impl<'a, C> ComposerModel<C>
where
    C: Clone,
    Dom<C>: ToHtml<C>,
    &'a str: ToHtml<C>,
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
    pub fn replace_text(&mut self, new_text: &[C]) -> ComposerUpdate<C> {
        // TODO: escape any HTML?
        let (s, e) = self.safe_selection();
        self.replace_text_in(&new_text, s, e)
    }

    /**
     * Replaces text in the an arbitrary start..end range with new_text.
     */
    pub fn replace_text_in(
        &mut self,
        new_text: &[C],
        start: usize,
        end: usize,
    ) -> ComposerUpdate<C> {
        // Store current Dom
        self.push_state_to_history();
        self.do_replace_text_in(new_text, start, end)
    }

    /// Internal: replace some text without modifying the undo/redo state.
    pub fn do_replace_text_in(
        &mut self,
        new_text: &[C],
        mut start: usize,
        end: usize,
    ) -> ComposerUpdate<C> {
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
                    .append(DomNode::Text(TextNode::from(new_text.to_vec())));

                start = 0;
            }
        }

        self.state.start = Location::from(start + new_text.len());
        self.state.end = self.state.start;

        // TODO: for now, we replace every time, to check ourselves, but
        // at least some of the time we should not
        self.create_update_replace_all()
    }

    pub fn backspace(&mut self) -> ComposerUpdate<C> {
        if self.state.start == self.state.end {
            // Go back 1 from the current location
            self.state.start -= 1;
        }

        self.replace_text(&[])
    }

    /**
     * Deletes text in an arbitrary start..end range.
     */
    pub fn delete_in(&mut self, start: usize, end: usize) -> ComposerUpdate<C> {
        self.state.end = Location::from(start);
        self.replace_text_in(&[], start, end)
    }

    /**
     * Deletes the character after the current cursor position.
     */
    pub fn delete(&mut self) -> ComposerUpdate<C> {
        if self.state.start == self.state.end {
            // Go forward 1 from the current location
            self.state.end += 1;
        }

        self.replace_text(&[])
    }

    pub fn action_response(
        &mut self,
        action_id: String,
        response: ActionResponse,
    ) -> ComposerUpdate<C> {
        drop(action_id);
        drop(response);
        ComposerUpdate::keep()
    }

    pub fn get_selection(&self) -> (Location, Location) {
        (self.state.start, self.state.end)
    }

    pub fn format(&mut self, format: InlineFormatType) -> ComposerUpdate<C> {
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
                self.state.dom.append(DomNode::new_formatting(
                    format.tag().to_html(),
                    vec![DomNode::Text(TextNode::from("".to_html()))],
                ));
                return ComposerUpdate::keep();
            }

            _ => panic!("Can't format in complex object models yet"),
        }
    }

    pub fn create_ordered_list(&mut self) -> ComposerUpdate<C> {
        self.create_list(true)
    }

    pub fn create_unordered_list(&mut self) -> ComposerUpdate<C> {
        self.create_list(false)
    }

    pub fn get_html(&self) -> Vec<C> {
        self.state.dom.to_html()
    }

    pub fn undo(&mut self) -> ComposerUpdate<C> {
        if let Some(prev) = self.previous_states.pop() {
            self.next_states.push(self.state.clone());
            self.state = prev;
            self.create_update_replace_all()
        } else {
            ComposerUpdate::keep()
        }
    }

    pub fn redo(&mut self) -> ComposerUpdate<C> {
        if let Some(next) = self.next_states.pop() {
            self.previous_states.push(self.state.clone());
            self.state = next;
            self.create_update_replace_all()
        } else {
            ComposerUpdate::keep()
        }
    }

    pub fn get_current_state(&self) -> &ComposerState<C> {
        &self.state
    }

    // Internal functions
    fn create_update_replace_all(&self) -> ComposerUpdate<C> {
        ComposerUpdate::replace_all(
            self.state.dom.to_html(),
            self.state.start,
            self.state.end,
        )
    }

    fn create_list(&mut self, ordered: bool) -> ComposerUpdate<C> {
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
                        list_tag.to_html(),
                        vec![DomNode::Container(ContainerNode::new_list_item(
                            "li".to_html(),
                            vec![DomNode::Text(TextNode::from(text.to_vec()))],
                        ))],
                    );
                    self.state.dom.replace(range.node_handle, vec![list_node]);
                    return self.create_update_replace_all();
                } else {
                    panic!("Can't create a list from a non-text node")
                }
            }

            Range::NoNode => {
                self.state.dom.append(DomNode::new_list(
                    list_tag.to_html(),
                    vec![DomNode::Container(ContainerNode::new_list_item(
                        "li".to_html(),
                        vec![DomNode::Text(TextNode::from("".to_html()))],
                    ))],
                ));
                return self.create_update_replace_all();
            }

            _ => {
                panic!("Can't create ordered list in complex object models yet")
            }
        }
    }

    fn replace_same_node(&mut self, range: SameNodeRange, new_text: &[C]) {
        let node = self.state.dom.lookup_node_mut(range.node_handle);
        if let DomNode::Text(ref mut t) = node {
            let text = t.data();
            let mut n = text[..range.start_offset].to_vec();
            n.extend_from_slice(new_text);
            n.extend_from_slice(&text[range.end_offset..]);
            t.set_data(n);
        } else {
            panic!("Can't deal with ranges containing non-text nodes (yet?)")
        }
    }

    fn replace_multiple_nodes(
        &mut self,
        range: MultipleNodesRange,
        new_text: &[C],
    ) {
        let mut first_text_node = true;
        let mut to_delete = Vec::new();
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
                        // Otherwise, replace the selected text
                        let mut new_data =
                            old_data[..loc.start_offset].to_vec();

                        if first_text_node {
                            new_data.extend_from_slice(new_text);
                        }

                        new_data.extend_from_slice(&old_data[loc.end_offset..]);
                        node.set_data(new_data);
                    }

                    first_text_node = false;
                }
            }
        }

        // Delete in reverse order to avoid invalidating handles
        to_delete.reverse();

        while !to_delete.is_empty() {
            let mut new_to_delete = Vec::new();

            for handle in to_delete.into_iter() {
                let child_index =
                    handle.raw().last().expect("Text node can't be root!");
                let parent_handle = handle.parent_handle();
                let mut parent =
                    self.state.dom.lookup_node_mut(parent_handle.clone());
                match &mut parent {
                    DomNode::Container(parent) => {
                        parent.remove(*child_index);
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
            let before = text[..range.start_offset].to_vec();
            let during = text[range.start_offset..range.end_offset].to_vec();
            let after = text[range.end_offset..].to_vec();
            let new_nodes = vec![
                DomNode::Text(TextNode::from(before)),
                DomNode::new_formatting(
                    format.tag().to_html(),
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
}

impl<'a> ComposerModel<u16>
where
    Dom<u16>: ToHtml<u16>,
    &'a str: ToHtml<u16>,
{
    pub fn enter(&mut self) -> ComposerUpdate<u16> {
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
                    self.replace_text(&"\n".encode_utf16().collect::<Vec<_>>())
                }
            }
            _ => self.replace_text(&"\n".encode_utf16().collect::<Vec<_>>()),
        }
    }

    pub fn replace_all_html(&mut self, html: &[u16]) -> ComposerUpdate<u16> {
        let dom = parse(&String::from_utf16(html).expect("Invalid UTF-16"));

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

    pub fn set_link(&mut self, link: Vec<u16>) -> ComposerUpdate<u16> {
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

    fn set_link_same_node(&mut self, range: SameNodeRange, link: Vec<u16>) {
        // TODO: set link should be able to wrap container nodes, unlike formatting
        let node = self.state.dom.lookup_node(range.node_handle.clone());
        if let DomNode::Text(t) = node {
            let text = t.data();
            // TODO: can we be globally smart about not leaving empty text nodes ?
            let before = text[..range.start_offset].to_vec();
            let during = text[range.start_offset..range.end_offset].to_vec();
            let after = text[range.end_offset..].to_vec();
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
            list.append(DomNode::new_list_item(
                "li".to_html(),
                vec![DomNode::Text(TextNode::from("\u{200B}".to_html()))],
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
                    parent.remove(handle.index_in_parent());
                    if parent.children().len() == 0 {
                        parent.append(DomNode::Text(TextNode::from(
                            "".to_html(),
                        )));
                    }
                    self.state.start = Location::from(location);
                    self.state.end = Location::from(location);
                } else {
                    // TODO: handle list items outside of lists
                    panic!("List has no parent container")
                }
            } else {
                list.remove(list_item_index);
                let parent_handle = handle.parent_handle();
                let parent_node = self.state.dom.lookup_node_mut(parent_handle);
                if let DomNode::Container(parent) = parent_node {
                    // TODO: should probably append a paragraph instead
                    parent.append(DomNode::Text(TextNode::from(
                        "\u{200B}".to_html(),
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
    use super::*;
    use crate::tests::testutils_composer_model::cm;

    use crate::dom::{DomHandle, ToHtml};

    // Most tests for ComposerModel are inside the tests/ modules

    #[test]
    fn completely_replacing_html_works() {
        let mut model = cm("{hello}| world");
        model.replace_all_html(&"foo <b>bar</b>".to_html());
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
