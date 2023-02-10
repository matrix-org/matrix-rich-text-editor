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
fn set_content_from_html() -> Result<(), DomCreationError> {
    let mut model = cm("|");
    model.set_content_from_html(&utf16("content"))?;
    assert_eq!(tx(&model), "content|");
    Ok(())
}

#[test]
fn set_content_from_markdown() -> Result<(), DomCreationError> {
    let mut model = cm("|");
    model.set_content_from_markdown(&utf16("**abc**"))?;
    assert_eq!(tx(&model), "<strong>abc|</strong>");
    Ok(())
}

#[test]
fn set_content_from_markdown_invalid() {
    let mut model = cm("|");
    let result = model
        .set_content_from_markdown(&utf16("`````"))
        .unwrap_err();
    assert_eq!(
        result,
        DomCreationError::HtmlParseError(HtmlParseError {
            parse_errors: vec!["Panic".into()]
        })
    );
}

#[test]
fn set_content_from_html_moves_cursor_to_the_end(
) -> Result<(), DomCreationError> {
    let mut model = cm("abc|");
    model.set_content_from_html(&"content".into())?;
    assert_eq!(tx(&model), "content|");
    Ok(())
}

#[test]
fn clear() -> Result<(), DomCreationError> {
    let mut model = cm("|");
    model.set_content_from_html(&Utf16String::from("content"))?;
    model.clear();
    assert_eq!(tx(&model), "|");
    Ok(())
}

#[test]
fn set_contents_with_line_break_in_code_block() {
    // The first line break inside a block node will be removed as it can be used to just give
    // structure to the node
    let model = cm("<pre>\n<code>|Test</code></pre>");
    assert_eq!(tx(&model), "<pre><code>|Test</code></pre>");
}
