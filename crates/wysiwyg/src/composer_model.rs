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

use crate::{
    ActionResponse, ByteLocation, CodepointDelta, CodepointLocation,
    ComposerUpdate,
};

pub struct ComposerModel {
    html: String, // TODO: not an AST yet!
    selection_start_codepoint: CodepointLocation,
    selection_end_codepoint: CodepointLocation,
}

impl ComposerModel {
    pub fn new() -> Self {
        Self {
            html: String::from(""),
            selection_start_codepoint: CodepointLocation::from(0),
            selection_end_codepoint: CodepointLocation::from(0),
        }
    }

    /**
     * Cursor is at end_codepoint.
     */
    pub fn select(
        &mut self,
        start_codepoint: CodepointLocation,
        end_codepoint: CodepointLocation,
    ) {
        self.selection_start_codepoint = start_codepoint;
        self.selection_end_codepoint = end_codepoint;
    }

    pub fn replace_text(&mut self, new_text: &str) -> ComposerUpdate {
        // TODO: escape any HTML?

        let range = &mut [
            self.selection_start_byte().as_usize(),
            self.selection_end_byte().as_usize(),
        ];
        range.sort();
        self.html.replace_range(range[0]..range[1], new_text);

        // Move cursor to after the inserted text
        let new_cp = self.selection_earliest_codepoint();
        self.selection_start_codepoint = ByteLocation::from(
            new_cp.byte(&self.html).as_usize() + new_text.len(),
        )
        .codepoint(&self.html);
        self.selection_end_codepoint = self.selection_start_codepoint;

        // TODO: for now, we replace every time, to check ourselves, but
        // at least some of the time we should not
        self.create_update_replace_all()
        //ComposerUpdate::keep()
    }

    pub fn enter(&mut self) -> ComposerUpdate {
        ComposerUpdate::keep()
    }

    pub fn backspace(&mut self) -> ComposerUpdate {
        if self.selection_start_codepoint == self.selection_end_codepoint {
            // Go back 1 from the current location
            self.selection_start_codepoint
                .move_forward(CodepointDelta::from(-1));
        }

        self.replace_text("")
    }

    pub fn delete(&mut self) -> ComposerUpdate {
        if self.selection_start_codepoint == self.selection_end_codepoint {
            // Go forward 1 from the current location
            self.selection_end_codepoint
                .move_forward(CodepointDelta::from(1));
        }

        self.replace_text("")
    }

    pub fn bold(&mut self) -> ComposerUpdate {
        let mut range = [
            self.selection_start_byte().as_usize(),
            self.selection_end_byte().as_usize(),
        ];
        range.sort();

        // TODO: not a real AST
        self.html = format!(
            "{}<strong>{}</strong>{}",
            &self.html[..range[0]],
            &self.html[range[0]..range[1]],
            &self.html[range[1]..]
        );

        /*
        TODO: probably requires a real AST
        let start_b = ByteLocation::from(range[0]);
        let end_b = ByteLocation::from(range[1] + "<strong></strong>".len());

        self.selection_start_codepoint = start_b.codepoint(&self.html);
        self.selection_end_codepoint = end_b.codepoint(&self.html);
        */

        self.create_update_replace_all()
    }

    pub fn action_response(
        &mut self,
        action_id: String,
        response: ActionResponse,
    ) -> ComposerUpdate {
        drop(action_id);
        drop(response);
        ComposerUpdate::keep()
    }

    // Internal functions

    fn create_update_replace_all(&self) -> ComposerUpdate {
        ComposerUpdate::replace_all(
            self.html.clone(),
            self.selection_start_codepoint,
            self.selection_end_codepoint,
        )
    }

    fn selection_start_byte(&self) -> ByteLocation {
        self.selection_start_codepoint.byte(&self.html)
    }

    fn selection_end_byte(&self) -> ByteLocation {
        self.selection_end_codepoint.byte(&self.html)
    }

    fn selection_earliest_codepoint(&self) -> CodepointLocation {
        if self.selection_start_codepoint.as_usize()
            <= self.selection_end_codepoint.as_usize()
        {
            self.selection_start_codepoint
        } else {
            self.selection_end_codepoint
        }
    }
}

#[cfg(test)]
mod test {
    use speculoos::{prelude::*, AssertionFailure, Spec};

    use crate::{ByteLocation, CodepointDelta, CodepointLocation};

    use super::ComposerModel;

    #[test]
    fn typing_a_character_into_an_empty_box_appends_it() {
        let mut model = cm("|");
        model.replace_text("v");
        assert_eq!(tx(model), "v|");
    }

    #[test]
    fn typing_a_character_at_the_end_appends_it() {
        let mut model = cm("abc|");
        model.replace_text("d");
        assert_eq!(tx(model), "abcd|");
    }

    #[test]
    fn typing_a_character_in_the_middle_inserts_it() {
        let mut model = cm("|abc");
        model.replace_text("Z");
        assert_eq!(tx(model), "Z|abc");
    }

    #[test]
    fn selecting_past_the_end_is_harmless() {
        let mut model = cm("|");
        model.select(CodepointLocation::from(7), CodepointLocation::from(7));
        model.replace_text("Z");
        assert_eq!(tx(model), "Z|");
    }

    #[test]
    fn replacing_a_selection_with_a_character() {
        let mut model = cm("abc{def}|ghi");
        model.replace_text("Z");
        assert_eq!(tx(model), "abcZ|ghi");
    }

    #[test]
    fn replacing_a_backwards_selection_with_a_character() {
        let mut model = cm("abc|{def}ghi");
        model.replace_text("Z");
        assert_eq!(tx(model), "abcZ|ghi");
    }

    #[test]
    fn typing_a_character_after_a_multi_codepoint_character() {
        // Woman Astronaut:
        // Woman+Dark Skin Tone+Zero Width Joiner+Rocket
        let mut model = cm("\u{1F469}\u{1F3FF}\u{200D}\u{1F680}|");
        model.replace_text("Z");
        assert_eq!(tx(model), "\u{1F469}\u{1F3FF}\u{200D}\u{1F680}Z|");
    }

    #[test]
    fn backspacing_a_character_at_the_end_deletes_it() {
        let mut model = cm("abc|");
        model.backspace();
        assert_eq!(tx(model), "ab|");
    }

    #[test]
    fn backspacing_a_character_at_the_beginning_does_nothing() {
        let mut model = cm("|abc");
        model.backspace();
        assert_eq!(tx(model), "|abc");
    }

    #[test]
    fn backspacing_a_character_in_the_middle_deletes_it() {
        let mut model = cm("ab|c");
        model.backspace();
        assert_eq!(tx(model), "a|c");
    }

    #[test]
    fn backspacing_a_selection_deletes_it() {
        let mut model = cm("a{bc}|");
        model.backspace();
        assert_eq!(tx(model), "a|");
    }

    #[test]
    fn backspacing_a_backwards_selection_deletes_it() {
        let mut model = cm("a|{bc}");
        model.backspace();
        assert_eq!(tx(model), "a|");
    }

    #[test]
    fn deleting_a_character_at_the_end_does_nothing() {
        let mut model = cm("abc|");
        model.delete();
        assert_eq!(tx(model), "abc|");
    }

    #[test]
    fn deleting_a_character_at_the_beginning_deletes_it() {
        let mut model = cm("|abc");
        model.delete();
        assert_eq!(tx(model), "|bc");
    }

    #[test]
    fn deleting_a_character_in_the_middle_deletes_it() {
        let mut model = cm("a|bc");
        model.delete();
        assert_eq!(tx(model), "a|c");
    }

    #[test]
    fn deleting_a_selection_deletes_it() {
        let mut model = cm("a{bc}|");
        model.delete();
        assert_eq!(tx(model), "a|");
    }

    #[test]
    fn deleting_a_backwards_selection_deletes_it() {
        let mut model = cm("a|{bc}");
        model.delete();
        assert_eq!(tx(model), "a|");
    }

    // Test utils

    trait Roundtrips<T> {
        fn roundtrips(&self);
    }

    impl<'s, T> Roundtrips<T> for Spec<'s, T>
    where
        T: AsRef<str>,
    {
        fn roundtrips(&self) {
            let subject = self.subject.as_ref();
            let output = tx(cm(subject));
            if tx(cm(subject)) != subject {
                AssertionFailure::from_spec(self)
                    .with_expected(String::from(subject))
                    .with_actual(output)
                    .fail();
            }
        }
    }

    /**
     * Create a ComposerModel from a text representation.
     */
    fn cm(text: &str) -> ComposerModel {
        let curs = text.find('|').expect(&format!(
            "ComposerModel text did not contain a '|' symbol: '{}'",
            text,
        ));
        let curs_b = ByteLocation::from(curs);
        let curs_cp = curs_b.codepoint(text);

        let s = text.find('{');
        let e = text.find('}');

        let mut ret = ComposerModel::new();
        if let (Some(s), Some(e)) = (s, e) {
            let s_b = ByteLocation::from(s);
            let e_b = ByteLocation::from(e);
            let s_cp = s_b.codepoint(text);
            let mut e_cp = e_b.codepoint(text);

            if curs == e + 1 {
                // Cursor after end: foo{bar}|baz
                // The { made an extra codepoint - move the end back 1
                e_cp.move_forward(CodepointDelta::from(-1));
                ret.selection_start_codepoint = s_cp;
                ret.selection_end_codepoint = e_cp;
                ret.html = String::from(&text[..s])
                    + &text[s + 1..e]
                    + &text[curs + 1..];
            } else if curs == s - 1 {
                // Cursor before beginning: foo|{bar}baz
                // The |{ made an extra 2 codepoints - move the end back 2
                e_cp.move_forward(CodepointDelta::from(-2));
                ret.selection_start_codepoint = e_cp;
                ret.selection_end_codepoint = curs_cp;
                ret.html = String::from(&text[..curs])
                    + &text[s + 1..e]
                    + &text[e + 1..];
            } else {
                panic!(
                    "The cursor ('|') must always be directly before or after \
                    the selection ('{{..}}')! \
                    E.g.: 'foo|{{bar}}baz' or 'foo{{bar}}|baz'."
                )
            }
        } else {
            ret.selection_start_codepoint = curs_cp;
            ret.selection_end_codepoint = curs_cp;
            ret.html = String::from(&text[..curs]) + &text[curs + 1..];
        }
        ret
    }

    /**
     * Convert a ComposerModel to a text representation.
     */
    fn tx(model: ComposerModel) -> String {
        let mut ret = model.html.clone();
        if model.selection_start_codepoint == model.selection_end_codepoint {
            ret.insert(model.selection_start_byte().as_usize(), '|');
        } else {
            let s = model.selection_start_byte().as_usize();
            let e = model.selection_end_byte().as_usize();
            if s < e {
                ret.insert_str(e, "}|");
                ret.insert_str(s, "{");
            } else {
                ret.insert_str(s, "}");
                ret.insert_str(e, "|{");
            }
        }
        ret
    }

    #[test]
    fn cm_creates_correct_component_model() {
        assert_eq!(cm("|").selection_start_codepoint.as_usize(), 0);
        assert_eq!(cm("|").selection_end_codepoint.as_usize(), 0);
        assert_eq!(cm("|").html, "");

        assert_eq!(cm("a|").selection_start_codepoint.as_usize(), 1);
        assert_eq!(cm("a|").selection_end_codepoint.as_usize(), 1);
        assert_eq!(cm("a|").html, "a");

        assert_eq!(cm("a|b").selection_start_codepoint.as_usize(), 1);
        assert_eq!(cm("a|b").selection_end_codepoint.as_usize(), 1);
        assert_eq!(cm("a|b").html, "ab");

        assert_eq!(cm("|ab").selection_start_codepoint.as_usize(), 0);
        assert_eq!(cm("|ab").selection_end_codepoint.as_usize(), 0);
        assert_eq!(cm("|ab").html, "ab");

        assert_eq!(cm("foo|").selection_start_codepoint.as_usize(), 3);
        assert_eq!(cm("foo|").selection_end_codepoint.as_usize(), 3);
        assert_eq!(cm("foo|").html, "foo");

        let t1 = cm("foo|\u{1F4A9}bar");
        assert_eq!(t1.selection_start_codepoint.as_usize(), 3);
        assert_eq!(t1.selection_end_codepoint.as_usize(), 3);
        assert_eq!(t1.html, "foo\u{1F4A9}bar");

        let t2 = cm("foo\u{1F4A9}|bar");
        assert_eq!(t2.selection_start_codepoint.as_usize(), 4);
        assert_eq!(t2.selection_end_codepoint.as_usize(), 4);
        assert_eq!(t2.html, "foo\u{1F4A9}bar");

        assert_eq!(cm("foo|\u{1F4A9}").selection_start_codepoint.as_usize(), 3);
        assert_eq!(cm("foo|\u{1F4A9}").selection_end_codepoint.as_usize(), 3);
        assert_eq!(cm("foo|\u{1F4A9}").html, "foo\u{1F4A9}");

        assert_eq!(cm("foo\u{1F4A9}|").selection_start_codepoint.as_usize(), 4);
        assert_eq!(cm("foo\u{1F4A9}|").selection_end_codepoint.as_usize(), 4);
        assert_eq!(cm("foo\u{1F4A9}|").html, "foo\u{1F4A9}");

        assert_eq!(cm("|\u{1F4A9}bar").selection_start_codepoint.as_usize(), 0);
        assert_eq!(cm("|\u{1F4A9}bar").selection_end_codepoint.as_usize(), 0);
        assert_eq!(cm("|\u{1F4A9}bar").html, "\u{1F4A9}bar");

        assert_eq!(cm("\u{1F4A9}|bar").selection_start_codepoint.as_usize(), 1);
        assert_eq!(cm("\u{1F4A9}|bar").selection_end_codepoint.as_usize(), 1);
        assert_eq!(cm("\u{1F4A9}|bar").html, "\u{1F4A9}bar");

        assert_eq!(cm("{a}|").selection_start_codepoint.as_usize(), 0);
        assert_eq!(cm("{a}|").selection_end_codepoint.as_usize(), 1);
        assert_eq!(cm("{a}|").html, "a");

        assert_eq!(cm("|{a}").selection_start_codepoint.as_usize(), 1);
        assert_eq!(cm("|{a}").selection_end_codepoint.as_usize(), 0);
        assert_eq!(cm("|{a}").html, "a");

        assert_eq!(cm("abc{def}|ghi").selection_start_codepoint.as_usize(), 3);
        assert_eq!(cm("abc{def}|ghi").selection_end_codepoint.as_usize(), 6);
        assert_eq!(cm("abc{def}|ghi").html, "abcdefghi");

        assert_eq!(cm("abc|{def}ghi").selection_start_codepoint.as_usize(), 6);
        assert_eq!(cm("abc|{def}ghi").selection_end_codepoint.as_usize(), 3);
        assert_eq!(cm("abc|{def}ghi").html, "abcdefghi");

        let t3 = cm("\u{1F4A9}{def}|ghi");
        assert_eq!(t3.selection_start_codepoint.as_usize(), 1);
        assert_eq!(t3.selection_end_codepoint.as_usize(), 4);
        assert_eq!(t3.html, "\u{1F4A9}defghi");

        let t4 = cm("\u{1F4A9}|{def}ghi");
        assert_eq!(t4.selection_start_codepoint.as_usize(), 4);
        assert_eq!(t4.selection_end_codepoint.as_usize(), 1);
        assert_eq!(t4.html, "\u{1F4A9}defghi");

        let t5 = cm("abc{d\u{1F4A9}f}|ghi");
        assert_eq!(t5.selection_start_codepoint.as_usize(), 3);
        assert_eq!(t5.selection_end_codepoint.as_usize(), 6);
        assert_eq!(t5.html, "abcd\u{1F4A9}fghi");

        let t6 = cm("abc|{d\u{1F4A9}f}ghi");
        assert_eq!(t6.selection_start_codepoint.as_usize(), 6);
        assert_eq!(t6.selection_end_codepoint.as_usize(), 3);
        assert_eq!(t6.html, "abcd\u{1F4A9}fghi");

        let t7 = cm("abc{def}|\u{1F4A9}ghi");
        assert_eq!(t7.selection_start_codepoint.as_usize(), 3);
        assert_eq!(t7.selection_end_codepoint.as_usize(), 6);
        assert_eq!(t7.html, "abcdef\u{1F4A9}ghi");

        let t8 = cm("abc|{def}\u{1F4A9}ghi");
        assert_eq!(t8.selection_start_codepoint.as_usize(), 6);
        assert_eq!(t8.selection_end_codepoint.as_usize(), 3);
        assert_eq!(t8.html, "abcdef\u{1F4A9}ghi");
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
    }
}
