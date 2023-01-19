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
    tests::testutils_composer_model::{cm, tx},
    ComposerModel,
};

#[test]
fn pressing_enter_with_a_brand_new_model() {
    let mut model = ComposerModel::new();
    model.enter();
    assert_eq!(tx(&model), "<br />|");
}

#[test]
fn pressing_enter_after_replacing_with_empty_html() {
    let mut model = ComposerModel::new();
    model.set_content_from_html(&Utf16String::new());
    model.enter();
    assert_eq!(tx(&model), "<br />|");
}

#[test]
fn pressing_enter_after_backspacing_a_line_break() {
    let mut model = cm("|");
    model.new_line();
    model.backspace();
    model.new_line();
    assert_eq!(tx(&model), "<p>|</p>");
}

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

#[test]
fn can_backspace_to_beginning_after_making_a_line() {
    let mut model = cm("a|");
    model.enter();
    model.backspace();
    model.backspace();
    assert_eq!(tx(&model), "|");
}

#[test]
fn test_replace_text_in_first_line_with_line_break() {
    let mut model = cm("{AAA}|<br />BBB");
    model.enter();
    assert_eq!(tx(&model), "<br />|<br />BBB");
}

#[test]
fn backspace_merges_text_nodes() {
    let mut model = cm("a<br />|b");
    model.backspace();
    assert_eq!(tx(&model), "a|b");
    // The two text nodes were merged
    assert_eq!(model.state.dom.document().children().len(), 1);
}

#[test]
fn backspace_merges_formatting_nodes() {
    let mut model = cm("<b>a</b><br />|<b>b</b>");
    model.backspace();
    assert_eq!(tx(&model), "<b>a|b</b>");
}

#[test]
fn enter_in_code_block_in_text_node_adds_line_break_as_text() {
    let mut model = cm("<pre>~Test|</pre>");
    model.enter();
    assert_eq!(tx(&model), "<pre>~Test\n|</pre>")
}

#[test]
fn enter_in_code_block_at_start_adds_the_line_break() {
    let mut model = cm("<pre>~|Test</pre>");
    model.enter();
    assert_eq!(tx(&model), "<pre>~\n|Test</pre>")
}

#[test]
fn enter_in_code_block_at_start_with_previous_line_break_moves_it_outside_the_code_block(
) {
    // The initial line break will be removed, so it's the same as having a single line break at the start
    let mut model = cm("|");
    model.code_block();
    model.replace_text("Test".into());
    model.select(0.into(), 0.into());
    assert_eq!(tx(&model), "<pre>|Test</pre>");
    model.new_line();
    assert_eq!(tx(&model), "<pre>\n|Test</pre>");
    model.select(0.into(), 0.into());
    model.new_line();
    assert_eq!(tx(&model), "<p>|</p><pre>Test</pre>");
}

#[test]
fn enter_in_code_block_at_start_with_previous_line_break_moves_it_outside_the_code_block_with_text_around(
) {
    // The initial line break will be removed, so it's the same as having a single line break at the start
    let mut model = cm("<p>ASDA</p><pre>\n|\nTest</pre><p>ASD</p>");
    model.new_line();
    assert_eq!(tx(&model), "<p>ASDA</p><p>|</p><pre>Test</pre><p>ASD</p>")
}

#[test]
fn enter_in_code_block_at_start_with_a_line_break_after_it_adds_another_one() {
    // The initial line break will be removed, so it's the same as having a single line break at the start
    let mut model = cm("<pre>\n~|\nTest</pre>");
    model.enter();
    assert_eq!(tx(&model), "<pre>~\n|\nTest</pre>")
}

#[test]
fn enter_in_code_block_after_line_break_in_middle_splits_code_block() {
    let mut model = cm("<pre>Test\n|\ncode blocks</pre>");
    model.new_line();
    assert_eq!(tx(&model), "<pre>Test</pre><p>|</p><pre>code blocks</pre>")
}

#[test]
fn enter_in_code_block_after_nested_line_break_in_middle_splits_code_block() {
    let mut model = cm("<pre><b><i>Test\n|\ncode blocks</i></b></pre>");
    model.new_line();
    assert_eq!(tx(&model), "<pre><b><i>Test</i></b></pre><p>|</p><pre><b><i>code blocks</i></b></pre>")
}

#[test]
fn enter_in_code_block_after_line_break_at_end_exits_it() {
    let mut model = cm("<pre><b>Bold</b> plain\n|</pre>");
    model.new_line();
    assert_eq!(tx(&model), "<pre><b>Bold</b> plain</pre><p>|</p>")
}

#[test]
fn simple_enter_in_quote_adds_new_paragraph() {
    let mut model = cm("<blockquote><p>Left|Right</p></blockquote>");
    model.new_line();
    assert_eq!(
        tx(&model),
        "<blockquote><p>Left</p><p>|Right</p></blockquote>"
    );
}

#[test]
fn double_enter_in_quote_exits_the_quote() {
    let mut model =
        cm("<blockquote><p>Left</p><p>|</p><p>Right</p></blockquote>");
    model.new_line();
    assert_eq!(
        tx(&model),
        "<blockquote><p>Left</p></blockquote><p>|</p><blockquote><p>Right</p></blockquote>"
    );
}

#[test]
fn double_enter_in_quote_at_start_when_empty() {
    let mut model = cm("<blockquote><p>|</p></blockquote>");
    model.new_line();
    assert_eq!(tx(&model), "<p>|</p>");
}

#[test]
fn double_enter_in_quote_at_start_when_not_empty() {
    let mut model = cm("<blockquote><p>|</p><p>Text</p></blockquote>");
    model.new_line();
    assert_eq!(tx(&model), "<p>|</p><blockquote><p>Text</p></blockquote>");
}

#[test]
fn double_enter_in_quote_at_end_when_not_empty_exits_it() {
    let mut model = cm("<blockquote><p>Text</p><p>|</p></blockquote>");
    model.new_line();
    assert_eq!(tx(&model), "<blockquote><p>Text</p></blockquote><p>|</p>");
}

#[test]
fn double_enter_in_code_block_when_empty_removes_it_and_adds_new_line() {
    let mut model = cm("|");
    model.code_block();
    assert_eq!(tx(&model), "<pre>|</pre>");
    model.new_line();
    assert_eq!(tx(&model), "<p>|</p>");
    model.replace_text("asd".into());
    assert_eq!(tx(&model), "<p>asd|</p>");
}

#[test]
fn double_enter_in_quote_in_nested_nodes() {
    let mut model = cm("\
        <blockquote>\
            <p><i><b>Left</b></i></p>\
            <p>|</p>\
            <p><b><i>Right</i></b></p>\
        </blockquote>");
    model.new_line();
    assert_eq!(
        tx(&model),
        "<blockquote>\
            <p><i><b>Left</b></i></p>\
        </blockquote>\
        <p>|</p>\
        <blockquote>\
            <p><b><i>Right</i></b></p>\
        </blockquote>"
    );
}

#[test]
fn backspace_at_start_of_code_block_moves_contents_to_previous_block() {
    let mut model = cm("<p>Test</p><pre>|code</pre>");
    model.backspace();
    assert_eq!(tx(&model), "<p>Test|code</p>");
}

#[test]
fn backspace_at_end_of_code_block_adds_the_content_of_the_next_block_node_to_it(
) {
    let mut model = cm("<p>Test</p><pre>code</pre><p>|and more</p>");
    model.backspace();
    assert_eq!(tx(&model), "<p>Test</p><pre>code|and more</pre>");
}

#[test]
fn backspace_emptying_code_block_removes_it() {
    let mut model = cm("<p>Test</p><pre>|</pre>");
    model.backspace();
    assert_eq!(tx(&model), "<p>Test|</p>");
}
