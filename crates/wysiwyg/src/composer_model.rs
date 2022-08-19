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
        let range = self.state.dom.find_range_mut(start, end);
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
        let range = self.state.dom.find_range_mut(s, e);
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

        let range = self.state.dom.find_range_mut(s, e);
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
    use crate::dom::nodes::{DomNode, TextNode};
    use crate::dom::ToHtml;
    use crate::tests::testutils::{cm, tx};
    use crate::InlineFormatType::Bold;
    use crate::{InlineFormatType, Location, TextUpdate};

    use super::ComposerModel;

    #[test]
    fn typing_a_character_into_an_empty_box_appends_it() {
        let mut model = cm("|");
        replace_text(&mut model, "v");
        assert_eq!(tx(&model), "v|");
    }

    #[test]
    fn typing_a_character_at_the_end_appends_it() {
        let mut model = cm("abc|");
        replace_text(&mut model, "d");
        assert_eq!(tx(&model), "abcd|");
    }

    #[test]
    fn typing_a_character_inside_a_tag_inserts_it() {
        let mut model = cm("AAA<b>BB|B</b>CCC");
        replace_text(&mut model, "Z");
        assert_eq!(tx(&model), "AAA<b>BBZ|B</b>CCC");
    }

    #[test]
    fn typing_a_character_in_the_middle_inserts_it() {
        let mut model = cm("|abc");
        replace_text(&mut model, "Z");
        assert_eq!(tx(&model), "Z|abc");
    }

    #[test]
    fn selecting_past_the_end_is_harmless() {
        let mut model = cm("|");
        model.select(Location::from(7), Location::from(7));
        replace_text(&mut model, "Z");
        assert_eq!(tx(&model), "Z|");
    }

    #[test]
    fn replacing_a_selection_with_a_character() {
        let mut model = cm("abc{def}|ghi");
        replace_text(&mut model, "Z");
        assert_eq!(tx(&model), "abcZ|ghi");
    }

    #[test]
    fn replacing_a_backwards_selection_with_a_character() {
        let mut model = cm("abc|{def}ghi");
        replace_text(&mut model, "Z");
        assert_eq!(tx(&model), "abcZ|ghi");
    }

    #[test]
    fn typing_a_character_after_a_multi_codepoint_character() {
        // Woman Astronaut:
        // Woman+Dark Skin Tone+Zero Width Joiner+Rocket
        let mut model = cm("\u{1F469}\u{1F3FF}\u{200D}\u{1F680}|");
        replace_text(&mut model, "Z");
        assert_eq!(tx(&model), "\u{1F469}\u{1F3FF}\u{200D}\u{1F680}Z|");
    }

    #[test]
    fn typing_a_character_in_a_range_inserts_it() {
        let mut model = cm("0123456789|");
        let new_text = "654".encode_utf16().collect::<Vec<u16>>();
        model.replace_text_in(&new_text, 4, 7);
        assert_eq!(tx(&model), "0123654|789");
    }

    #[test]
    fn backspacing_a_character_at_the_end_deletes_it() {
        let mut model = cm("abc|");
        model.backspace();
        assert_eq!(tx(&model), "ab|");
    }

    #[test]
    fn backspacing_a_character_at_the_beginning_does_nothing() {
        let mut model = cm("|abc");
        model.backspace();
        assert_eq!(tx(&model), "|abc");
    }

    #[test]
    fn backspacing_a_character_in_the_middle_deletes_it() {
        let mut model = cm("ab|c");
        model.backspace();
        assert_eq!(tx(&model), "a|c");
    }

    #[test]
    fn backspacing_a_selection_deletes_it() {
        let mut model = cm("a{bc}|");
        model.backspace();
        assert_eq!(tx(&model), "a|");
    }

    #[test]
    fn backspacing_a_backwards_selection_deletes_it() {
        let mut model = cm("a|{bc}");
        model.backspace();
        assert_eq!(tx(&model), "a|");
    }

    #[test]
    fn deleting_a_character_at_the_end_does_nothing() {
        let mut model = cm("abc|");
        model.delete();
        assert_eq!(tx(&model), "abc|");
    }

    #[test]
    fn deleting_a_character_at_the_beginning_deletes_it() {
        let mut model = cm("|abc");
        model.delete();
        assert_eq!(tx(&model), "|bc");
    }

    #[test]
    fn deleting_a_character_in_the_middle_deletes_it() {
        let mut model = cm("a|bc");
        model.delete();
        assert_eq!(tx(&model), "a|c");
    }

    #[test]
    fn deleting_a_selection_deletes_it() {
        let mut model = cm("a{bc}|");
        model.delete();
        assert_eq!(tx(&model), "a|");
    }

    #[test]
    fn deleting_a_backwards_selection_deletes_it() {
        let mut model = cm("a|{bc}");
        model.delete();
        assert_eq!(tx(&model), "a|");
    }

    #[test]
    fn deleting_a_range_removes_it() {
        let mut model = cm("abcd|");
        model.delete_in(1, 3);
        assert_eq!(tx(&model), "a|d");
    }

    #[test]
    fn selecting_ascii_characters() {
        let mut model = cm("abcdefgh|");
        model.select(Location::from(0), Location::from(1));
        assert_eq!(tx(&model), "{a}|bcdefgh");

        model.select(Location::from(1), Location::from(3));
        assert_eq!(tx(&model), "a{bc}|defgh");

        model.select(Location::from(4), Location::from(8));
        assert_eq!(tx(&model), "abcd{efgh}|");
    }

    // TODO: Test selecting invalid ranges, including starting and ending off
    // the end.

    #[test]
    fn selecting_single_utf16_code_unit_characters() {
        let mut model = cm("\u{03A9}\u{03A9}\u{03A9}|");

        model.select(Location::from(0), Location::from(1));
        assert_eq!(tx(&model), "{\u{03A9}}|\u{03A9}\u{03A9}");

        model.select(Location::from(0), Location::from(3));
        assert_eq!(tx(&model), "{\u{03A9}\u{03A9}\u{03A9}}|");

        model.select(Location::from(1), Location::from(2));
        assert_eq!(tx(&model), "\u{03A9}{\u{03A9}}|\u{03A9}");
    }

    #[test]
    fn selecting_multiple_utf16_code_unit_characters() {
        let mut model = cm("\u{1F4A9}\u{1F4A9}\u{1F4A9}|");

        model.select(Location::from(0), Location::from(2));
        assert_eq!(tx(&model), "{\u{1F4A9}}|\u{1F4A9}\u{1F4A9}");

        model.select(Location::from(0), Location::from(6));
        assert_eq!(tx(&model), "{\u{1F4A9}\u{1F4A9}\u{1F4A9}}|");

        model.select(Location::from(2), Location::from(4));
        assert_eq!(tx(&model), "\u{1F4A9}{\u{1F4A9}}|\u{1F4A9}");
    }

    #[test]
    fn selecting_complex_characters() {
        let mut model =
            cm("aaa\u{03A9}bbb\u{1F469}\u{1F3FF}\u{200D}\u{1F680}ccc|");

        model.select(Location::from(0), Location::from(3));
        assert_eq!(
            tx(&model),
            "{aaa}|\u{03A9}bbb\u{1F469}\u{1F3FF}\u{200D}\u{1F680}ccc"
        );

        model.select(Location::from(0), Location::from(4));
        assert_eq!(
            tx(&model),
            "{aaa\u{03A9}}|bbb\u{1F469}\u{1F3FF}\u{200D}\u{1F680}ccc"
        );

        model.select(Location::from(7), Location::from(14));
        assert_eq!(
            tx(&model),
            "aaa\u{03A9}bbb{\u{1F469}\u{1F3FF}\u{200D}\u{1F680}}|ccc"
        );

        model.select(Location::from(7), Location::from(15));
        assert_eq!(
            tx(&model),
            "aaa\u{03A9}bbb{\u{1F469}\u{1F3FF}\u{200D}\u{1F680}c}|cc"
        );
    }

    #[test]
    fn selecting_and_bolding_multiple_times() {
        let mut model = cm("aabbcc|");
        model.select(Location::from(0), Location::from(2));
        model.format(InlineFormatType::Bold);
        model.select(Location::from(4), Location::from(6));
        model.format(InlineFormatType::Bold);
        assert_eq!(
            &model.state.dom.to_string(),
            "<strong>aa</strong>bb<strong>cc</strong>"
        );
    }

    #[test]
    fn bolding_ascii_adds_strong_tags() {
        let mut model = cm("aa{bb}|cc");
        model.format(InlineFormatType::Bold);
        assert_eq!(tx(&model), "aa<strong>{bb}|</strong>cc");

        let mut model = cm("aa|{bb}cc");
        model.format(InlineFormatType::Bold);
        assert_eq!(tx(&model), "aa<strong>|{bb}</strong>cc");
    }

    #[test]
    fn undoing_action_restores_previous_state() {
        let mut model = cm("hello |");
        let mut prev = model.state.clone();
        let prev_text_node =
            TextNode::from("world!".encode_utf16().collect::<Vec<u16>>());
        prev.dom.append(DomNode::Text(prev_text_node));
        model.previous_states.push(prev.clone());

        model.undo();

        assert_eq!(prev.dom.children().len(), model.state.dom.children().len());
    }

    #[test]
    fn inserting_text_creates_previous_state() {
        let mut model = cm("|");
        assert!(model.previous_states.is_empty());

        replace_text(&mut model, "hello world!");
        assert!(!model.previous_states.is_empty());
    }

    #[test]
    fn backspacing_text_creates_previous_state() {
        let mut model = cm("hello world!|");
        assert!(model.previous_states.is_empty());

        model.backspace();
        assert!(!model.previous_states.is_empty());
    }

    #[test]
    fn deleting_text_creates_previous_state() {
        let mut model = cm("hello |world!");
        assert!(model.previous_states.is_empty());

        model.delete();
        assert!(!model.previous_states.is_empty());
    }

    #[test]
    fn formatting_text_creates_previous_state() {
        let mut model = cm("hello {world}|!");
        assert!(model.previous_states.is_empty());

        model.format(Bold);
        assert!(!model.previous_states.is_empty());
    }

    #[test]
    fn undoing_action_removes_last_previous_state() {
        let mut model = cm("hello {world}|!");
        model.previous_states.push(model.state.clone());

        model.undo();

        assert!(model.previous_states.is_empty());
    }

    #[test]
    fn undoing_action_adds_popped_state_to_next_states() {
        let mut model = cm("hello {world}|!");
        model.previous_states.push(model.state.clone());

        model.undo();

        assert_eq!(model.next_states[0], model.state);
    }

    #[test]
    fn redo_pops_state_from_next_states() {
        let mut model = cm("hello {world}|!");
        model.next_states.push(model.state.clone());

        model.redo();

        assert!(model.next_states.is_empty());
    }

    #[test]
    fn redoing_action_adds_popped_state_to_previous_states() {
        let mut model = cm("hello {world}|!");
        model.next_states.push(model.state.clone());

        model.redo();

        assert_eq!(model.previous_states[0], model.state);
    }

    #[test]
    fn cant_set_link_to_empty_selection() {
        let mut model = cm("hello |world");
        let update =
            model.set_link("https://element.io".encode_utf16().collect());
        assert!(matches!(update.text_update, TextUpdate::Keep));
    }

    #[test]
    fn set_link_wraps_selection_in_link_tag() {
        let mut model = cm("{hello}| world");
        model.set_link("https://element.io".encode_utf16().collect());
        assert_eq!(
            model.state.dom.to_string(),
            "<a href=\"https://element.io\">hello</a> world"
        );
    }

    #[test]
    fn completely_replacing_html_works() {
        let mut model = cm("{hello}| world");
        model.replace_all_html(&"foo <b>bar</b>".to_html());
        assert_eq!(model.state.dom.to_string(), "foo <b>bar</b>");
    }

    #[test]
    fn can_replace_text_in_an_empty_composer_model() {
        let mut cm = ComposerModel::new();
        cm.replace_text(&"foo".to_html());
        assert_eq!(tx(&cm), "foo|");
    }

    // Test utils

    fn replace_text(model: &mut ComposerModel<u16>, new_text: &str) {
        model.replace_text(&new_text.encode_utf16().collect::<Vec<u16>>());
    }
}
