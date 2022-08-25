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

use crate::{ComposerModel, ToRawText};

#[test]
fn test_ordered_list() {
    let mut model = cm("|");
    model.create_ordered_list();
    assert_eq!(tx(&model), "<ol><li>|</li></ol>");
    replace_text(&mut model, "abcd");
    assert_eq!(tx(&model), "<ol><li>abcd|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>abcd</li><li>\u{200b}|</li></ol>");
    replace_text(&mut model, "efgh");
    assert_eq!(tx(&model), "<ol><li>abcd</li><li>\u{200b}efgh|</li></ol>");
    assert_eq!(&model.state.dom.to_raw_text(), "abcd\u{200b}efgh");
}

#[test]
fn test_unordered_list() {
    let mut model = cm("|");
    model.create_unordered_list();
    assert_eq!(tx(&model), "<ul><li>|</li></ul>");
}

#[test]
fn test_removing_list_item() {
    let mut model = cm("<ol><li>abcd</li><li>\u{200b}|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>abcd</li></ol>\u{200b}|");
}

#[test]
fn test_removing_list() {
    let mut model = cm("|");
    model.create_ordered_list();
    model.enter();
    assert_eq!(tx(&model), "|");
}

fn replace_text(model: &mut ComposerModel<Utf16String>, new_text: &str) {
    model.replace_text(utf16(new_text));
}
