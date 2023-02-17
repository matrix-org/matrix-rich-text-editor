// Copyright 2023 The Matrix.org Foundation C.I.C.
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

use crate::tests::testutils_composer_model::{cm, tx};

// This file defines specific tests on how lists behave in combination to
// some other block nodes such as quote and code blocks. At some point, some
// of these behaviours might change with updates such as e.g. quotes
// available inside list items.

#[test]
fn create_list_inside_quote() {
    let mut model = cm("<blockquote><p>a|</p></blockquote>");
    model.ordered_list();
    assert_eq!(tx(&model), "<blockquote><ol><li>a|</li></ol></blockquote>")
}

#[test]
fn create_list_inside_quote_with_multiple_paragraphs() {
    let mut model =
        cm("<blockquote><p>{a</p><p>b</p><p>c}|</p><p>d</p></blockquote>");
    model.ordered_list();
    assert_eq!(
        tx(&model),
        "<blockquote><ol><li>{a</li><li>b</li><li>c}|</li></ol><p>d</p></blockquote>"
    )
}

#[test]
fn create_list_with_selected_paragraph_and_quote() {
    let mut model = cm("<p>{text</p><blockquote><p>quote}|</p></blockquote>");
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>{text</li><li>quote}|</li></ul>")
}

#[test]
fn create_list_with_selected_paragraph_and_quote_with_multiple_nested_paragraphs(
) {
    let mut model = cm(
        "<p>{text</p><blockquote><p>quote</p><p>more quote}|</p></blockquote>",
    );
    model.unordered_list();
    assert_eq!(
        tx(&model),
        "<ul><li>{text</li><li>quote</li><li>more quote}|</li></ul>"
    )
}

#[test]
fn create_list_with_selected_paragraph_and_codeblock() {
    let mut model = cm("<p>{text</p><pre><code>some code}|</code></pre>");
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>{text</li><li>some code}|</li></ul>")
}

#[test]
fn create_list_with_selected_paragraph_and_codeblock_with_multiple_lines() {
    let mut model = cm("<p>{text</p><pre><code>code\nmore code}|</code></pre>");
    model.unordered_list();
    assert_eq!(
        tx(&model),
        "<ul><li>{text</li><li>code</li><li>more code}|</li></ul>"
    )
}

#[test]
fn create_list_with_selected_paragraph_quote_and_code_block() {
    let mut model = cm("<blockquote><p>{quote</p><p>more quote</p></blockquote><p>text</p><pre><code>code\nmore code}|</code></pre>");
    model.ordered_list();
    assert_eq!(
        tx(&model),
        "<ol><li>{quote</li><li>more quote</li><li>text</li><li>code</li><li>more code}|</li></ol>"
    )
}
