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
    ComposerModel, ToHtml,
};

#[test]
#[allow(deprecated)]
fn pressing_enter_with_a_brand_new_model() {
    let mut model = ComposerModel::new();
    model.add_line_break();
    assert_eq!(tx(&model), "<br />|");
}

#[test]
#[allow(deprecated)]
fn adding_line_break_after_replacing_with_empty_html() {
    let mut model = ComposerModel::new();
    model.set_content_from_html(&Utf16String::new()).unwrap();
    model.add_line_break();
    assert_eq!(tx(&model), "<br />|");
}

#[test]
fn pressing_enter_after_backspacing_a_paragraph() {
    let mut model = cm("|");
    model.enter();
    assert_eq!(tx(&model), "<p>&nbsp;</p><p>&nbsp;|</p>");
    model.backspace();
    assert_eq!(tx(&model), "<p>&nbsp;|</p>");
    model.enter();
    assert_eq!(tx(&model), "<p>&nbsp;</p><p>&nbsp;|</p>");
}

#[test]
#[allow(deprecated)]
fn pressing_enter_with_an_empty_model_inserts_a_line_break() {
    let mut model = cm("|");
    model.add_line_break();
    assert_eq!(tx(&model), "<br />|");
}

#[test]
#[allow(deprecated)]
fn adding_line_break_at_the_beginning_of_a_line_makes_a_new_line_above() {
    let mut model = cm("|abc");
    model.add_line_break();
    assert_eq!(tx(&model), "<br />|abc");
}

#[test]
#[allow(deprecated)]
fn adding_line_break_at_the_end_of_a_line_makes_a_new_line() {
    let mut model = cm("abc|");
    model.add_line_break();
    assert_eq!(tx(&model), "abc<br />|");
}

#[test]
#[allow(deprecated)]
fn adding_line_break_in_the_middle_of_a_line_splits_it() {
    let mut model = cm("123|abc");
    model.add_line_break();
    assert_eq!(tx(&model), "123<br />|abc");
}

#[test]
#[allow(deprecated)]
fn adding_line_break_with_text_selected_splits_the_line() {
    let mut model = cm("123{XYZ}|abc");
    model.add_line_break();
    assert_eq!(tx(&model), "123<br />|abc");
}

#[test]
#[allow(deprecated)]
fn multiple_line_breaks_can_be_added() {
    let mut model = cm("123|abc");
    model.add_line_break();
    model.add_line_break();
    model.add_line_break();
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
#[allow(deprecated)]
fn can_add_line_break_on_later_lines() {
    let mut model = cm("asd<br />sad|");
    model.add_line_break();
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
#[allow(deprecated)]
fn type_after_adding_line_break() {
    let mut model = cm("a|");
    model.add_line_break();
    model.replace_text(Utf16String::from_str("b"));
    assert_eq!(tx(&model), "a<br />b|");
}

#[test]
#[allow(deprecated)]
fn can_backspace_to_beginning_after_adding_a_line_break() {
    let mut model = cm("a|");
    model.add_line_break();
    model.backspace();
    model.backspace();
    assert_eq!(tx(&model), "|");
}

#[test]
#[allow(deprecated)]
fn test_replace_text_in_first_line_with_line_break() {
    let mut model = cm("{AAA}|<br />BBB");
    model.add_line_break();
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
    let mut model = cm("<pre><code>Test|</code></pre>");
    model.enter();
    assert_eq!(tx(&model), "<pre><code>Test\n&nbsp;|</code></pre>")
}

#[test]
fn enter_in_code_block_at_start_adds_the_line_break() {
    let mut model = cm("<pre><code>|Test</code></pre>");
    model.enter();
    assert_eq!(tx(&model), "<pre><code>&nbsp;\n|Test</code></pre>")
}

#[test]
fn enter_in_code_block_at_start_with_previous_line_break_moves_it_outside_the_code_block(
) {
    // The initial line break will be removed, so it's the same as having a single line break at the start
    let mut model = cm("|");
    model.code_block();
    model.replace_text("Test".into());
    model.select(0.into(), 0.into());
    assert_eq!(tx(&model), "<pre><code>|Test</code></pre>");
    model.enter();
    assert_eq!(tx(&model), "<pre><code>&nbsp;\n|Test</code></pre>");
    model.select(0.into(), 0.into());
    model.enter();
    assert_eq!(tx(&model), "<p>&nbsp;|</p><pre><code>Test</code></pre>");
}

#[test]
fn enter_in_code_block_at_start_with_previous_line_break_moves_it_outside_the_code_block_with_text_around(
) {
    let mut model = cm("<p>ASDA</p><pre><code>|\nTest</code></pre><p>ASD</p>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<p>ASDA</p><p>&nbsp;|</p><pre><code>Test</code></pre><p>ASD</p>"
    )
}

#[test]
fn enter_in_code_block_at_start_with_a_line_break_after_it_adds_another_one() {
    let mut model = cm("<pre><code>\n\n|Test</code></pre>");
    model.enter();
    assert_eq!(tx(&model), "<pre><code>&nbsp;\n\n\n|Test</code></pre>")
}

#[test]
fn enter_in_code_block_after_line_break_in_middle_splits_code_block() {
    let mut model = cm("<pre><code>Test\n|\ncode blocks</code></pre>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<pre><code>Test</code></pre><p>&nbsp;|</p><pre><code>code blocks</code></pre>"
    )
}

#[test]
fn enter_in_code_block_after_nested_line_break_in_middle_splits_code_block() {
    let mut model =
        cm("<pre><code><b><i>Test\n|\ncode blocks</i></b></code></pre>");
    model.enter();
    assert_eq!(tx(&model), "<pre><code><b><i>Test</i></b></code></pre><p>&nbsp;|</p><pre><code><b><i>code blocks</i></b></code></pre>")
}

#[test]
fn enter_in_code_block_after_line_break_at_end_exits_it() {
    let mut model = cm("<pre><code><b>Bold</b> plain\n|</code></pre>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<pre><code><b>Bold</b> plain</code></pre><p>&nbsp;|</p>"
    )
}

#[test]
fn simple_enter_in_quote_adds_new_paragraph() {
    let mut model = cm("<blockquote><p>Left|Right</p></blockquote>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<blockquote><p>Left</p><p>|Right</p></blockquote>"
    );
}

#[test]
fn double_enter_in_quote_exits_the_quote() {
    let mut model =
        cm("<blockquote><p>Left</p><p>|</p><p>Right</p></blockquote>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<blockquote><p>Left</p></blockquote><p>&nbsp;|</p><blockquote><p>Right</p></blockquote>"
    );
}

#[test]
fn double_enter_in_quote_at_start_when_empty() {
    let mut model = cm("<blockquote><p>|</p></blockquote>");
    model.enter();
    assert_eq!(tx(&model), "<p>&nbsp;|</p>");
}

#[test]
fn double_enter_in_quote_at_start_when_not_empty() {
    let mut model = cm("<blockquote><p>|</p><p>Text</p></blockquote>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<p>&nbsp;|</p><blockquote><p>Text</p></blockquote>"
    );
}

#[test]
fn double_enter_in_quote_at_end_when_not_empty_exits_it() {
    let mut model = cm("<blockquote><p>Text</p><p>|</p></blockquote>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<blockquote><p>Text</p></blockquote><p>&nbsp;|</p>"
    );
}

#[test]
fn double_enter_in_code_block_when_empty_removes_it_and_adds_new_line() {
    let mut model = cm("|");
    model.code_block();
    assert_eq!(tx(&model), "<pre><code>&nbsp;|</code></pre>");
    model.enter();
    assert_eq!(tx(&model), "<p>&nbsp;|</p>");
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
    model.enter();
    assert_eq!(
        tx(&model),
        "<blockquote>\
            <p><i><b>Left</b></i></p>\
        </blockquote>\
        <p>&nbsp;|</p>\
        <blockquote>\
            <p><b><i>Right</i></b></p>\
        </blockquote>"
    );
}

#[test]
fn backspace_at_start_of_code_block_moves_contents_to_previous_block() {
    let mut model = cm("<p>Test</p><pre><code>|code</code></pre>");
    model.backspace();
    assert_eq!(tx(&model), "<p>Test|code</p>");
}

#[test]
fn backspace_at_end_of_code_block_adds_the_content_of_the_next_block_node_to_it(
) {
    let mut model =
        cm("<p>Test</p><pre><code>code</code></pre><p>|and more</p>");
    model.backspace();
    assert_eq!(
        tx(&model),
        "<p>Test</p><pre><code>code|and more</code></pre>"
    );
}

#[test]
fn backspace_emptying_code_block_removes_it() {
    let mut model = cm("<p>Test</p><pre><code>|</code></pre>");
    model.backspace();
    assert_eq!(tx(&model), "<p>Test|</p>");
}

#[test]
fn text_typed_after_line_break_goes_into_last_paragraph() {
    let mut model = cm("|");
    model.enter();
    model.select(1.into(), 1.into());
    model.replace_text("Test".into());
    assert_eq!(tx(&model), "<p>&nbsp;</p><p>Test|</p>");
}

#[test]
fn backspace_after_several_empty_paragraphs_deletes_only_one() {
    let mut model = cm("<p></p><p></p><p>|</p>");
    model.backspace();
    assert_eq!(tx(&model), "<p>&nbsp;</p><p>&nbsp;|</p>");
}

#[test]
fn new_line_at_start_of_link_does_not_extend_it() {
    let mut model = cm("<b><a href='test'>|Test</a></b>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<p>&nbsp;</p>\
        <p>\
            <b>\
                <a href=\"test\">|Test</a>\
            </b>\
        </p>"
    );
}

#[test]
fn new_line_at_end_of_link_does_not_extend_it() {
    let mut model = cm("<b><a href='test'>Test|</a></b>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<p>\
            <b>\
                <a href=\"test\">Test</a>\
            </b>\
        </p>\
        <p>\
            <b>|</b>\
        </p>"
    );
}

#[test]
fn pressing_enter_after_wrapping_text_in_code_block_works() {
    let mut model = cm("|");
    model.replace_text("Some code".into());
    model.code_block();
    model.enter();
    assert_eq!(tx(&model), "<pre><code>Some code\n&nbsp;|</code></pre>")
}

#[test]
fn pressing_enter_at_the_start_of_a_multiline_code_block() {
    let mut model = cm("<pre><code>|line_1\nline_2</code></pre>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<pre><code>&nbsp;\n|line_1\nline_2</code></pre>"
    )
}

#[test]
fn pressing_enter_at_the_start_of_a_multiline_block_quote() {
    let mut model = cm("<blockquote><p>|line_1</p><p>line_2</p></blockquote>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<blockquote><p>&nbsp;</p><p>|line_1</p><p>line_2</p></blockquote>"
    )
}

#[test]
fn pressing_enter_in_the_middle_of_a_multiline_code_block_with_empty_starting_paragraphs(
) {
    let mut model = cm("<pre><code>|line_1\nline_2</code></pre>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<pre><code>&nbsp;\n|line_1\nline_2</code></pre>"
    )
}

#[test]
fn pressing_enter_in_the_middle_of_a_multiline_code_block() {
    let mut model = cm("<pre><code>line_0\n|line_1\nline_2</code></pre>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<pre><code>line_0\n\n|line_1\nline_2</code></pre>"
    )
}

#[test]
fn pressing_enter_in_the_middle_of_a_multiline_block_quote_with_empty_starting_paragraphs(
) {
    let mut model =
        cm("<blockquote><p></p><p>|line_1</p><p>line_2</p></blockquote>");
    model.enter();
    assert_eq!(tx(&model), "<blockquote><p>&nbsp;</p><p>&nbsp;</p><p>|line_1</p><p>line_2</p></blockquote>")
}

#[test]
fn pressing_enter_in_the_middle_of_a_multiline_block_quote() {
    let mut model =
        cm("<blockquote><p>line_0</p><p>|line_1</p><p>line_2</p></blockquote>");
    model.enter();
    assert_eq!(tx(&model), "<blockquote><p>line_0</p><p>&nbsp;</p><p>|line_1</p><p>line_2</p></blockquote>")
}
