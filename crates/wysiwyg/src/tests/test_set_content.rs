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

use crate::{
    dom::DomCreationError,
    tests::{testutils_composer_model::tx, testutils_conversion::utf16},
    HtmlParseError,
};

use super::testutils_composer_model::cm;

#[test]
fn set_content_from_html() {
    let mut model = cm("|");
    model.set_content_from_html(&utf16("content")).unwrap();
    assert_eq!(tx(&model), "content|");
}

#[test]
fn set_content_from_html_invalid() {
    let mut model = cm("|");
    let error = model
        .set_content_from_html(&utf16("<strong>hello<strong>"))
        .unwrap_err();
    assert_eq!(
        error,
        DomCreationError::HtmlParseError(HtmlParseError::new(vec![
            "Unexpected open tag at end of body".into()
        ]))
    );
}

#[test]
fn set_content_from_markdown() {
    let mut model = cm("|");
    model.set_content_from_markdown(&utf16("**abc**")).unwrap();
    assert_eq!(tx(&model), "<strong>abc|</strong>");
}

#[test]
fn set_content_from_html_moves_cursor_to_the_end() {
    let mut model = cm("abc|");
    model.set_content_from_html(&"content".into()).unwrap();
    assert_eq!(tx(&model), "content|");
}

#[test]
fn clear() {
    let mut model = cm("|");
    model
        .set_content_from_html(&Utf16String::from("content"))
        .unwrap();
    model.clear();
    assert_eq!(tx(&model), "|");
}

#[test]
fn set_contents_with_line_break_in_code_block() {
    // The first line break inside a block node will be removed as it can be used to just give
    // structure to the node
    let model = cm("<pre>\n<code>|Test</code></pre>");
    assert_eq!(tx(&model), "<pre><code>|Test</code></pre>");
}
