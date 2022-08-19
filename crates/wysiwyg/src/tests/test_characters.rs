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

use crate::tests::testutils::{cm, tx};

use crate::{ComposerModel, Location, ToHtml};

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
fn typing_a_character_in_a_range_inserts_it() {
    let mut model = cm("0123456789|");
    let new_text = "654".encode_utf16().collect::<Vec<u16>>();
    model.replace_text_in(&new_text, 4, 7);
    assert_eq!(tx(&model), "0123654|789");
}

#[test]
fn can_replace_text_in_an_empty_composer_model() {
    let mut cm = ComposerModel::new();
    cm.replace_text(&"foo".to_html());
    assert_eq!(tx(&cm), "foo|");
}

fn replace_text(model: &mut ComposerModel<u16>, new_text: &str) {
    model.replace_text(&new_text.to_html());
}
