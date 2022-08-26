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

#![cfg(test)]

use widestring::Utf16String;

use crate::tests::testutils_composer_model::{cm, tx};
use crate::tests::testutils_conversion::utf16;

use crate::{ComposerModel, Location};

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
fn replacing_a_selection_past_the_end_is_harmless() {
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
fn replacing_an_explicit_text_range_works() {
    let mut model = cm("0123456789|");
    let new_text = utf16("654");
    model.replace_text_in(new_text, 4, 7);
    assert_eq!(tx(&model), "0123654|789");
}

#[test]
fn can_replace_text_in_an_empty_composer_model() {
    let mut cm = ComposerModel::new();
    cm.replace_text(utf16("foo"));
    assert_eq!(tx(&cm), "foo|");
}

#[test]
fn typing_a_character_when_spanning_two_tags_extends_the_first_tag() {
    let mut model = cm("before<b>bo{ld</b>aft}|er");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "before<b>boZ|</b>er");
}

#[test]
fn replacing_an_explicit_range_when_spanning_two_tags_extends_the_first_tag() {
    let mut model = cm("|before<b>bold</b>after");
    model.replace_text_in(utf16("XYZ"), 8, 13);
    assert_eq!(tx(&model), "before<b>boXYZ|</b>er");
}

#[test]
fn typing_a_character_when_spanning_two_whole_tags_extends_the_first_tag() {
    let mut model = cm("before<b>{bold</b>after}|");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "before<b>Z|</b>");
}

#[test]
fn typing_a_character_when_spanning_entire_tag_keeps_formatting() {
    let mut model = cm("before<b>{bo<i>x</i>ld}|</b>after");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "before<b>Z|</b>after");
}

#[test]
fn typing_a_character_when_spanning_over_newly_opened_tags_deletes_them() {
    let mut model = cm("before<b>bo{ld</b>a<i>f</i>t}|er");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "before<b>boZ|</b>er");
}

#[test]
fn typing_when_spanning_multiple_close_tags_extends_the_first_tag() {
    let mut model = cm("00<code><i>2<b>33{33</b></i>55</code>6}|6");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "00<code><i>2<b>33Z|</b></i></code>6");
}

#[test]
fn typing_when_spanning_open_tags_moves_their_start_forwards() {
    let mut model = cm("0{0<b>1<i>2}|2</i>3</b>44");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "0Z|<b><i>2</i>3</b>44");
}

#[test]
fn typing_that_empties_an_end_tag_deletes_it() {
    let mut model = cm("00{00<b>1111}|</b>");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "00Z|");
}

#[test]
fn typing_when_spanning_whole_open_tags_moves_their_start_forwards() {
    let mut model = cm("{00<b>1<i>22}|</i>3</b>44");
    replace_text(&mut model, "Z");
    assert_eq!(tx(&model), "Z|<b>3</b>44");
}

fn replace_text(model: &mut ComposerModel<Utf16String>, new_text: &str) {
    model.replace_text(utf16(new_text));
}
