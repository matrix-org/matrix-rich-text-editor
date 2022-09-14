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

#[test]
fn pressing_enter_with_an_empty_model_inserts_a_line_break() {
    let mut model = cm("|");
    model.enter();
    assert_eq!(tx(&model), "<br />|");
}

#[test]
fn pressing_enter_at_the_beginning_of_a_line_makes_a_new_line_above() {
    let mut model = cm("|abc");
    model.enter();
    assert_eq!(tx(&model), "<br />|abc");
}

#[test]
fn pressing_enter_at_the_end_of_a_line_makes_a_new_line() {
    let mut model = cm("abc|");
    model.enter();
    assert_eq!(tx(&model), "abc<br />|");
}

#[test]
fn pressing_enter_in_the_middle_of_a_line_splits_it() {
    let mut model = cm("123|abc");
    model.enter();
    assert_eq!(tx(&model), "123<br />|abc");
}

#[test]
fn pressing_enter_with_text_selected_splits_the_line() {
    let mut model = cm("123{XYZ}|abc");
    model.enter();
    assert_eq!(tx(&model), "123<br />|abc");
}

#[test]
fn multiple_enters_make_new_lines_each_time() {
    let mut model = cm("123|abc");
    model.enter();
    model.enter();
    model.enter();
    assert_eq!(tx(&model), "123<br /><br /><br />|abc");
}

#[test]
fn can_place_cursor_inside_brs_and_delete() {
    let mut model = cm("123<br />|<br />abc");
    model.backspace();
    assert_eq!(tx(&model), "123|<br />abc");

    let mut model = cm("123<br />|<br />abc");
    model.delete();
    assert_eq!(tx(&model), "123<br />|abc");
}

#[test]
fn can_press_enter_on_later_lines() {
    let mut model = cm("asd<br />sad|");
    model.enter();
    // Doesn't crash
}

#[test]
fn backspace_to_beginning_of_line() {
    let mut model = cm("123<br />a|bc");
    model.backspace();
    assert_eq!(tx(&model), "123<br />|bc");
}

#[test]
fn backspace_deletes_br() {
    let mut model = cm("123<br />|abc");
    model.backspace();
    assert_eq!(tx(&model), "123|abc");
}

#[test]
fn delete_deletes_br() {
    let mut model = cm("123|<br />abc");
    model.delete();
    assert_eq!(tx(&model), "123|abc");
}

#[test]
fn type_after_pressing_enter() {
    let mut model = cm("a|");
    model.enter();
    model.replace_text(Utf16String::from_str("b"));
    assert_eq!(tx(&model), "a<br />b|");
}

#[ignore] // TODO: can_backspace_to_beginning_after_making_a_line
#[test]
fn can_backspace_to_beginning_after_making_a_line() {
    let mut model = cm("a|");
    model.enter();
    model.backspace();
    model.backspace();
    assert_eq!(tx(&model), "|");
}

#[ignore] // TODO: backspace_merges_empty_text_nodes
#[test]
fn backspace_merges_empty_text_nodes() {
    let mut model = cm("a<br />|");
    model.backspace();
    assert_eq!(tx(&model), "a|");
    // The two text nodes were merged
    assert_eq!(model.state.dom.document().children().len(), 1);
}

#[ignore] // TODO: backspace_merges_text_nodes
#[test]
fn backspace_merges_text_nodes() {
    let mut model = cm("a<br />|b");
    model.backspace();
    assert_eq!(tx(&model), "a|b");
    // The two text nodes were merged
    assert_eq!(model.state.dom.document().children().len(), 1);
}

#[ignore] // TODO: backspace_merges_formatting_nodes
#[test]
fn backspace_merges_formatting_nodes() {
    let mut model = cm("<b>a</b><br />|<b>b</b>");
    model.backspace();
    assert_eq!(tx(&model), "<b>a|b</b>");
}
