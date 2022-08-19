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
use crate::dom::parser::parse;
use crate::dom::{Dom, Range, SameNodeRange, ToHtml};
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
    pub(crate) fn do_replace_text_in(
        &mut self,
        new_text: &[C],
        start: usize,
        end: usize,
    ) -> ComposerUpdate<C> {
        let range = self.state.dom.find_range(start, end);
        match range {
            Range::SameNode(range) => {
                self.replace_same_node(range, new_text);
                self.state.start = Location::from(start + new_text.len());
                self.state.end = self.state.start;
            }

            Range::NoNode => {
                self.state
                    .dom
                    .append(DomNode::Text(TextNode::from(new_text.to_vec())));

                self.state.start = Location::from(new_text.len());
                self.state.end = self.state.start;
            }

            _ => panic!(
                "Can't replace_text_in in complex object models yet. {:?}",
                range
            ),
        }

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

    pub fn enter(&mut self) -> ComposerUpdate<C> {
        ComposerUpdate::keep()
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
}

#[cfg(test)]
mod test {
    use crate::tests::testutils::cm;

    use crate::dom::ToHtml;

    // Most tests for ComposerModel are inside the tests/ modules

    #[test]
    fn completely_replacing_html_works() {
        let mut model = cm("{hello}| world");
        model.replace_all_html(&"foo <b>bar</b>".to_html());
        assert_eq!(model.state.dom.to_string(), "foo <b>bar</b>");
    }
}
