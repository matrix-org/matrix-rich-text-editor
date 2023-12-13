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

use indoc::indoc;
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
fn set_content_from_html_containing_newlines() {
    let mut model = cm("|");
    model
        .set_content_from_html(&utf16(
            "<p> \n <strong> \n \n Hello world! \n \n </strong> \n \n </p> \n\n\n",
        ))
        .unwrap();
    assert_eq!(
        &model.to_tree(),
        indoc! {
        r#"

        └>p
          └>strong
            └>"Hello world!"
        "#}
    );
    assert_eq!(tx(&model), "<p><strong>Hello world!|</strong></p>");
}

#[test]
fn set_content_from_html_paragraphs() {
    let mut model = cm("|");
    model
        .set_content_from_html(&utf16(
            "<p>\n  paragraph 1\n</p>\n<p> \n  paragraph 2\n</p>",
        ))
        .unwrap();
    assert_eq!(
        &model.to_tree(),
        indoc! {
        r#"

        ├>p
        │ └>"paragraph 1"
        └>p
          └>"paragraph 2"
        "#}
    );
    assert_eq!(tx(&model), "<p>paragraph 1</p><p>paragraph 2|</p>");
}

#[test]
fn set_content_from_html_paragraphs_containing_newline() {
    let mut model = cm("|");
    model
        .set_content_from_html(&utf16(
            "<p>\n  paragraph\n  across two lines\n</p>\n",
        ))
        .unwrap();
    assert_eq!(
        &model.to_tree(),
        indoc! {
        r#"

        └>p
          └>"paragraph across two lines"
        "#}
    );
    assert_eq!(tx(&model), "<p>paragraph across two lines|</p>");
}

#[test]
fn set_content_from_html_paragraphs_and_inline() {
    let mut model = cm("|");
    model
        .set_content_from_html(&utf16(
            "<p>\n  paragraph 1\n</p>\n<b>\n  inline\n</b>\n<p>\n  paragraph 2\n</p>",
        ))
        .unwrap();
    assert_eq!(
        &model.to_tree(),
        indoc! {
        r#"

        ├>p
        │ └>"paragraph 1"
        ├>p
        │ └>b
        │   └>"inline"
        └>p
          └>"paragraph 2"
        "#}
    );
    assert_eq!(
        tx(&model),
        "<p>paragraph 1</p><p><b>inline</b></p><p>paragraph 2|</p>"
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

#[test]
fn set_content_from_markdown_blockquote() {
    let mut model = cm("|");
    model.set_content_from_markdown(&utf16("> quote")).unwrap();
    assert_eq!(tx(&model), "<blockquote><p>quote|</p></blockquote>");
}

#[test]
fn set_content_from_markdown_blockquote_multiline() {
    let mut model = cm("|");
    model
        .set_content_from_markdown(&utf16("> quote\n\nfollowing text"))
        .unwrap();
    assert_eq!(
        tx(&model),
        "<blockquote><p>quote</p></blockquote><p>following text|</p>"
    );
}

#[test]
fn set_content_from_markdown_codeblock_with_newlines() {
    let mut model = cm("|");
    model
        .set_content_from_markdown(&utf16("```\nI am a code block\n```"))
        .unwrap();
    assert_eq!(tx(&model), "<pre><code>I am a code block|</code></pre>");
}

#[test]
fn set_content_from_markdown_multiple_new_lines() {
    let mut model = cm("|");
    model
        .set_content_from_markdown(&utf16("test\n\ntest"))
        .unwrap();
    assert_eq!(tx(&model), "<p>test</p><p>test|</p>");
}
