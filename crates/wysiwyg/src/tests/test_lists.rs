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

use crate::ComposerModel;

#[test]
fn creating_ordered_list_and_writing() {
    let mut model = cm("|");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>|</li></ol>");
    replace_text(&mut model, "abcd");
    assert_eq!(tx(&model), "<ol><li>abcd|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>abcd</li><li>\u{200b}|</li></ol>");
    replace_text(&mut model, "efgh");
    assert_eq!(tx(&model), "<ol><li>abcd</li><li>\u{200b}efgh|</li></ol>");
}

#[test]
fn creating_unordered_list() {
    let mut model = cm("|");
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>|</li></ul>");
}

#[test]
fn removing_list_item() {
    let mut model = cm("<ol><li>abcd</li><li>\u{200b}|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>abcd</li></ol>\u{200b}|");

    let mut model = cm("<ol><li>abcd</li><li>\u{200b}|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "<ol><li>abcd|</li></ol>");

    let mut model = cm("<ol><li>|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "|");

    let mut model = cm("<ol><li>|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "|");
}

#[test]
fn backspacing_in_list() {
    let mut model = cm("<ol><li>abcd</li><li>\u{200b}ef{gh}|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "<ol><li>abcd</li><li>\u{200b}ef|</li></ol>");

    let mut model = cm("<ol><li>ab{cd</li><li>\u{200b}efgh}|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "<ol><li>ab|</li></ol>");

    let mut model = cm("<ol><li>{abcd</li><li>\u{200b}}|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "<ol><li>|</li></ol>");
}

#[test]
fn entering_with_entire_selection() {
    let mut model = cm("<ol><li>{abcd}|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "|");

    let mut model = cm("<ol><li>{abcd</li><li>\u{200b}}|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "|");
}

#[test]
fn entering_with_entire_selection_with_formatting() {
    let mut model = cm("<ol><li><b>{abcd}|</b></li></ol>");
    model.enter();
    assert_eq!(tx(&model), "|");
}

#[test]
fn entering_mid_text_node() {
    let mut model = cm("<ol><li>ab|gh</li></ol>");
    model.enter();
    // FIXME: selection should be before the first char of second node
    assert_eq!(tx(&model), "<ol><li>ab|</li><li>gh</li></ol>");
}

#[test]
fn entering_mid_text_node_with_selection() {
    let mut model = cm("<ol><li>ab{cdef}|gh</li></ol>");
    model.enter();
    // FIXME: selection should be before the first char of second node
    assert_eq!(tx(&model), "<ol><li>ab|</li><li>gh</li></ol>");
}

#[test]
fn removing_list() {
    let mut model = cm("|");
    model.ordered_list();
    model.enter();
    assert_eq!(tx(&model), "|");
}

#[test]
fn updating_list_type() {
    let mut model = cm("<ol><li>ab</li><li>cd|</li></ol>");
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>ab</li><li>cd|</li></ul>");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>ab</li><li>cd|</li></ol>");
}

#[test]
fn moving_list_item_content_out() {
    let mut model = cm("<ol><li>ab</li><li>cd|</li></ol>");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>ab</li></ol>cd|");
}

#[test]
fn appending_new_list_to_previous() {
    let mut model = cm("<ol><li>ab</li></ol>cd|");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>ab</li><li>cd|</li></ol>");
}

#[test]
fn creating_list_of_different_type_doesnt_merge() {
    let mut model = cm("<ul><li>foo</li></ul>bar|");
    model.ordered_list();
    assert_eq!(tx(&model), "<ul><li>foo</li></ul><ol><li>bar|</li></ol>");
}

fn replace_text(model: &mut ComposerModel<Utf16String>, new_text: &str) {
    model.replace_text(utf16(new_text));
}
