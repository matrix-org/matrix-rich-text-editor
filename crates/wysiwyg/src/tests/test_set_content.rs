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

use widestring::Utf16String;

use crate::{tests::testutils_composer_model::tx, ComposerModel};

use super::testutils_composer_model::cm;

#[test]
fn set_content_from_html() {
    let mut model = ComposerModel::new();
    model.set_content_from_html(&Utf16String::from("content"));
    assert_eq!(tx(&model), "content|");
}

#[test]
fn set_content_from_markdown() {
    let mut model = ComposerModel::new();
    model.set_content_from_markdown(&Utf16String::from("**abc**"));
    assert_eq!(tx(&model), "<strong>abc|</strong>");
}

#[test]
fn set_content_from_html_moves_cursor_to_the_end() {
    let mut model = cm("abc|");
    model.set_content_from_html(&"content".into());
    assert_eq!(tx(&model), "content|");
}

#[test]
fn clear() {
    let mut model = ComposerModel::new();
    model.set_content_from_html(&Utf16String::from("content"));
    model.clear();
    assert_eq!(tx(&model), "|");
}
