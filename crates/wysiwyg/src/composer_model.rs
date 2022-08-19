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
    state: ComposerState<C>,
    previous_states: Vec<ComposerState<C>>,
    next_states: Vec<ComposerState<C>>,
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
    fn do_replace_text_in(
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
    use speculoos::{prelude::*, AssertionFailure, Spec};

    use crate::dom::nodes::{DomNode, TextNode};
    use crate::dom::parser::parse;
    use crate::dom::{Range, SameNodeRange, ToHtml};
    use crate::ComposerState;
    use crate::InlineFormatType::Bold;
    use crate::{InlineFormatType, Location, TextUpdate};

    use super::ComposerModel;

    fn utf8(utf16: &[u16]) -> String {
        String::from_utf16(&utf16).expect("Invalid UTF-16!")
    }

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

    // Test utils

    fn replace_text(model: &mut ComposerModel<u16>, new_text: &str) {
        model.replace_text(&new_text.encode_utf16().collect::<Vec<u16>>());
    }

    trait Roundtrips<T> {
        fn roundtrips(&self);
    }

    impl<'s, T> Roundtrips<T> for Spec<'s, T>
    where
        T: AsRef<str>,
    {
        fn roundtrips(&self) {
            let subject = self.subject.as_ref();
            let output = tx(&cm(subject));
            if output != subject {
                AssertionFailure::from_spec(self)
                    .with_expected(String::from(subject))
                    .with_actual(output)
                    .fail();
            }
        }
    }

    // TODO: too many tests and utils - split them up.

    /**
     * Create a ComposerModel from a text representation.
     */
    fn cm(text: &str) -> ComposerModel<u16> {
        let text_u16: Vec<u16> = text.encode_utf16().collect();

        fn find(haystack: &[u16], needle: &str) -> Option<usize> {
            let mut skip_count = 0; // How many tag characters we have seen
            let mut in_tag = false; // Are we in a tag now?

            let needle = needle.to_html()[0];
            let open = "<".to_html()[0];
            let close = ">".to_html()[0];

            for (i, &ch) in haystack.iter().enumerate() {
                if ch == needle {
                    return Some(i - skip_count);
                } else if ch == open {
                    in_tag = true;
                } else if ch == close {
                    skip_count += 1;
                    in_tag = false;
                }
                if in_tag {
                    skip_count += 1;
                }
            }
            None
        }

        let curs = find(&text_u16, "|").expect(&format!(
            "ComposerModel text did not contain a '|' symbol: '{}'",
            String::from_utf16(&text_u16)
                .expect("ComposerModel text was not UTF-16"),
        ));

        let s = find(&text_u16, "{");
        let e = find(&text_u16, "}");

        let mut model = ComposerModel {
            state: ComposerState::new(),
            previous_states: Vec::new(),
            next_states: Vec::new(),
        };
        model.state.dom = parse(&text).unwrap();

        fn delete_range(model: &mut ComposerModel<u16>, p1: usize, p2: usize) {
            model.do_replace_text_in(&[], p1, p2);
        }

        if let (Some(s), Some(e)) = (s, e) {
            if curs == e + 1 {
                // Cursor after end: foo{bar}|baz
                // The { made an extra codeunit - move the end back 1
                delete_range(&mut model, s, s + 1);
                delete_range(&mut model, e - 1, e + 1);
                model.state.start = Location::from(s);
                model.state.end = Location::from(e - 1);
            } else if curs == s - 1 {
                // Cursor before beginning: foo|{bar}baz
                // The |{ made an extra 2 codeunits - move the end back 2
                delete_range(&mut model, s - 1, s + 1);
                delete_range(&mut model, e - 2, e - 1);
                model.state.start = Location::from(e - 2);
                model.state.end = Location::from(curs);
            } else {
                panic!(
                    "The cursor ('|') must always be directly before or after \
                    the selection ('{{..}}')! \
                    E.g.: 'foo|{{bar}}baz' or 'foo{{bar}}|baz'."
                );
            }
        } else {
            delete_range(&mut model, curs, curs + 1);
            model.state.start = Location::from(curs);
            model.state.end = Location::from(curs);
        }

        model
    }

    /**
     * Convert a ComposerModel to a text representation.
     */
    fn tx(model: &ComposerModel<u16>) -> String {
        fn update_text_node_with_cursor(
            text_node: &mut TextNode<u16>,
            range: SameNodeRange,
        ) {
            let orig_s: usize = range.start_offset.into();
            let orig_e: usize = range.end_offset.into();
            let (s, e) = if orig_s < orig_e {
                (orig_s, orig_e)
            } else {
                (orig_e, orig_s)
            };

            let data = text_node.data();
            let mut new_data;
            if s == e {
                new_data = utf8(&data[..s]);
                new_data.push('|');
                new_data += &utf8(&data[s..]);
            } else {
                new_data = utf8(&data[..s]);
                if orig_s < orig_e {
                    new_data.push('{');
                } else {
                    new_data += "|{";
                }
                new_data += &utf8(&data[s..e]);
                if orig_s < orig_e {
                    new_data += "}|";
                } else {
                    new_data.push('}');
                }
                new_data += &utf8(&data[e..]);
            }
            text_node.set_data(new_data.to_html());
        }

        // Clone the model because we will modify it to add selection markers
        let mut model = model.clone();

        let range = model
            .state
            .dom
            .find_range_mut(model.state.start.into(), model.state.end.into());

        match range {
            Range::SameNode(range) => {
                let node =
                    model.state.dom.lookup_node_mut(range.node_handle.clone());
                match node {
                    DomNode::Container(_) => {
                        panic!("Don't know how to tx in a non-text node")
                    }
                    DomNode::Text(text_node) => {
                        update_text_node_with_cursor(text_node, range)
                    }
                }
            }
            Range::NoNode => panic!("No node!"),
            Range::TooDifficultForMe => {
                dbg!((model.state.start, model.state.end));
                todo!("Range too difficult!")
            }
        }

        model.state.dom.to_string()
    }

    #[test]
    fn can_replace_text_in_an_empty_composer_model() {
        let mut cm = ComposerModel::new();
        cm.replace_text(&"foo".to_html());
        assert_eq!(tx(&cm), "foo|");
    }

    #[test]
    fn cm_creates_correct_component_model() {
        // TODO: can we split and/or make these tests clearer?
        assert_eq!(cm("|").state.start, 0);
        assert_eq!(cm("|").state.end, 0);
        assert_eq!(cm("|").get_html(), &[]);

        assert_eq!(cm("a|").state.start, 1);
        assert_eq!(cm("a|").state.end, 1);
        assert_eq!(cm("a|").get_html(), "a".to_html());

        assert_eq!(cm("a|b").state.start, 1);
        assert_eq!(cm("a|b").state.end, 1);
        assert_eq!(cm("a|b").get_html(), "ab".to_html());

        assert_eq!(cm("|ab").state.start, 0);
        assert_eq!(cm("|ab").state.end, 0);
        assert_eq!(cm("|ab").get_html(), "ab".to_html());

        assert_eq!(cm("foo|").state.start, 3);
        assert_eq!(cm("foo|").state.end, 3);
        assert_eq!(cm("foo|").get_html(), ("foo".to_html()));

        let t0 = cm("AAA<b>B|BB</b>CCC");
        assert_eq!(t0.state.start, 4);
        assert_eq!(t0.state.end, 4);
        assert_eq!(t0.get_html(), "AAA<b>BBB</b>CCC".to_html());

        let t1 = cm("foo|\u{1F4A9}bar");
        assert_eq!(t1.state.start, 3);
        assert_eq!(t1.state.end, 3);
        assert_eq!(t1.get_html(), ("foo\u{1F4A9}bar").to_html());

        let t2 = cm("foo\u{1F4A9}|bar");
        assert_eq!(t2.state.start, 5);
        assert_eq!(t2.state.end, 5);
        assert_eq!(t2.get_html(), ("foo\u{1F4A9}bar").to_html());

        assert_eq!(cm("foo|\u{1F4A9}").state.start, 3);
        assert_eq!(cm("foo|\u{1F4A9}").state.end, 3);
        assert_eq!(cm("foo|\u{1F4A9}").get_html(), ("foo\u{1F4A9}").to_html());

        assert_eq!(cm("foo\u{1F4A9}|").state.start, 5);
        assert_eq!(cm("foo\u{1F4A9}|").state.end, 5);
        assert_eq!(cm("foo\u{1F4A9}|").get_html(), ("foo\u{1F4A9}").to_html());

        assert_eq!(cm("|\u{1F4A9}bar").state.start, 0);
        assert_eq!(cm("|\u{1F4A9}bar").state.end, 0);
        assert_eq!(cm("|\u{1F4A9}bar").get_html(), ("\u{1F4A9}bar").to_html());

        assert_eq!(cm("\u{1F4A9}|bar").state.start, 2);
        assert_eq!(cm("\u{1F4A9}|bar").state.end, 2);
        assert_eq!(cm("\u{1F4A9}|bar").get_html(), ("\u{1F4A9}bar").to_html());

        assert_eq!(cm("{a}|").state.start, 0);
        assert_eq!(cm("{a}|").state.end, 1);
        assert_eq!(cm("{a}|").get_html(), ("a").to_html());

        assert_eq!(cm("|{a}").state.start, 1);
        assert_eq!(cm("|{a}").state.end, 0);
        assert_eq!(cm("|{a}").get_html(), ("a").to_html());

        assert_eq!(cm("abc{def}|ghi").state.start, 3);
        assert_eq!(cm("abc{def}|ghi").state.end, 6);
        assert_eq!(cm("abc{def}|ghi").get_html(), ("abcdefghi").to_html());

        assert_eq!(cm("abc|{def}ghi").state.start, 6);
        assert_eq!(cm("abc|{def}ghi").state.end, 3);
        assert_eq!(cm("abc|{def}ghi").get_html(), ("abcdefghi").to_html());

        let t3 = cm("\u{1F4A9}{def}|ghi");
        assert_eq!(t3.state.start, 2);
        assert_eq!(t3.state.end, 5);
        assert_eq!(t3.get_html(), ("\u{1F4A9}defghi").to_html());

        let t4 = cm("\u{1F4A9}|{def}ghi");
        assert_eq!(t4.state.start, 5);
        assert_eq!(t4.state.end, 2);
        assert_eq!(t4.get_html(), ("\u{1F4A9}defghi").to_html());

        let t5 = cm("abc{d\u{1F4A9}f}|ghi");
        assert_eq!(t5.state.start, 3);
        assert_eq!(t5.state.end, 7);
        assert_eq!(t5.get_html(), ("abcd\u{1F4A9}fghi").to_html());

        let t6 = cm("abc|{d\u{1F4A9}f}ghi");
        assert_eq!(t6.state.start, 7);
        assert_eq!(t6.state.end, 3);
        assert_eq!(t6.get_html(), ("abcd\u{1F4A9}fghi").to_html());

        let t7 = cm("abc{def}|\u{1F4A9}ghi");
        assert_eq!(t7.state.start, 3);
        assert_eq!(t7.state.end, 6);
        assert_eq!(t7.get_html(), ("abcdef\u{1F4A9}ghi").to_html());

        let t8 = cm("abc|{def}\u{1F4A9}ghi");
        assert_eq!(t8.state.start, 6);
        assert_eq!(t8.state.end, 3);
        assert_eq!(t8.get_html(), ("abcdef\u{1F4A9}ghi").to_html());
    }

    #[test]
    fn cm_and_tx_roundtrip() {
        assert_that!("|").roundtrips();
        assert_that!("a|").roundtrips();
        assert_that!("a|b").roundtrips();
        assert_that!("|ab").roundtrips();
        assert_that!("foo|\u{1F4A9}bar").roundtrips();
        assert_that!("foo\u{1F4A9}|bar").roundtrips();
        assert_that!("foo|\u{1F4A9}").roundtrips();
        assert_that!("foo\u{1F4A9}|").roundtrips();
        assert_that!("|\u{1F4A9}bar").roundtrips();
        assert_that!("\u{1F4A9}|bar").roundtrips();
        assert_that!("{a}|").roundtrips();
        assert_that!("|{a}").roundtrips();
        assert_that!("abc{def}|ghi").roundtrips();
        assert_that!("abc|{def}ghi").roundtrips();
        assert_that!("\u{1F4A9}{def}|ghi").roundtrips();
        assert_that!("\u{1F4A9}|{def}ghi").roundtrips();
        assert_that!("abc{d\u{1F4A9}f}|ghi").roundtrips();
        assert_that!("abc|{d\u{1F4A9}f}ghi").roundtrips();
        assert_that!("abc{def}|\u{1F4A9}ghi").roundtrips();
        assert_that!("abc|{def}\u{1F4A9}ghi").roundtrips();
        assert_that!("AAA<b>B|BB</b>CCC").roundtrips();
        assert_that!("AAA<b>{BBB}|</b>CCC").roundtrips();
        assert_that!("AAA<b>|{BBB}</b>CCC").roundtrips();
        assert_that!("AAA<b>B<em>B|</em>B</b>CCC").roundtrips();
        assert_that!("AAA<b>B|<em>B</em>B</b>CCC").roundtrips();
        assert_that!("AAA<b>B<em>{B}|</em>B</b>CCC").roundtrips();
        assert_that!("AAA<b>B<em>|{B}</em>B</b>CCC").roundtrips();
        assert_that!("AAA<b>B<em>B</em>B</b>C|CC").roundtrips();
        assert_that!("AAA<b>B<em>B</em>B</b>{C}|CC").roundtrips();
        assert_that!("AAA<b>B<em>B</em>B</b>|{C}CC").roundtrips();
    }
}
