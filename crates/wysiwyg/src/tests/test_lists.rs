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
    assert_eq!(tx(&model), "<ol><li>abcd</li><li>|</li></ol>");
    replace_text(&mut model, "efgh");
    assert_eq!(tx(&model), "<ol><li>abcd</li><li>efgh|</li></ol>");
}

#[test]
fn creating_unordered_list() {
    let mut model = cm("|");
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>|</li></ul>");
}

#[test]
fn can_create_list_in_empty_model() {
    let mut model = ComposerModel::new();
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>|</li></ul>");
}

#[test]
fn removing_list_item() {
    let mut model = cm("<ol><li>abcd</li><li>|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>abcd</li></ol><p>&nbsp;|</p>");

    let mut model = cm("<ol><li>abcd</li><li>|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "<ol><li>abcd|</li></ol>");

    let mut model = cm("<ol><li>|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<p>&nbsp;|</p>");

    let mut model = cm("<ol><li>|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "|");
}

#[test]
fn backspacing_in_list_leaving_some_text_keeps_items_unchanged() {
    let mut model = cm("<ol><li>~abcd</li><li>~ef{gh}|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "<ol><li>~abcd</li><li>~ef|</li></ol>");
}

#[test]
fn backspacing_whole_of_second_item_list_into_part_of_first_leaves_one_item() {
    let mut model = cm("<ol><li>~ab{cd</li><li>~efgh}|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "<ol><li>~ab|</li></ol>");
}

#[test]
fn backspacing_empty_second_list_item_into_whole_of_first_leaves_empty_item() {
    let mut model = cm("<ol><li>~{abcd</li><li>~}|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "<ol><li>~|</li></ol>");
}

#[test]
fn backspacing_trailing_part_of_a_list_item() {
    let mut model =
        cm("<ol><li>~abc{def}|</li><li><strong>~abcd</strong>ef</li></ol>");
    model.backspace();
    assert_eq!(
        tx(&model),
        "<ol><li>~abc|</li><li><strong>~abcd</strong>ef</li></ol>"
    )
}

#[test]
fn backspacing_the_whole_list() {
    let mut model = cm("\
        <p>Text{</p>\
        <ul>\
            <li>First</li>\
            <li>Second}|</li>\
        </ul>\
        <p>More text</p>");
    model.backspace();
    assert_eq!(tx(&model), "<p>Text|</p><p>More text</p>");
}

#[test]
fn backspacing_through_several_list_items() {
    let mut model = cm("<p>Text</p><ul><li>Fi{rst</li><li>Seco}|nd</li></ul>");
    model.backspace();
    assert_eq!(tx(&model), "<p>Text</p><ul><li>Fi|nd</li></ul>");
}

#[test]
fn backspacing_at_start_of_first_list_item_adds_it_to_previous_block_node_if_exists(
) {
    let mut model = cm("<p>Text</p><ul><li>|First</li><li>Second</li></ul>");
    model.backspace();
    assert_eq!(tx(&model), "<p>Text|First</p><ul><li>Second</li></ul>");
}

#[test]
fn backspacing_at_start_of_first_list_item_extracts_it_if_no_previous_block_node_exists(
) {
    let mut model = cm("<ul><li>|First</li><li>Second</li></ul>");
    model.backspace();
    assert_eq!(tx(&model), "<p>|First</p><ul><li>Second</li></ul>");
}

#[test]
// #[ignore] // TODO: fix replace_text for cases where a list item contains formatting
fn backspacing_trailing_part_of_a_list_item_with_formatting() {
    let mut model = cm("<ol><li><strong>abc{d</strong>ef}|</li><li><strong>abcd</strong>ef</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "<ol><li><strong>abc|</strong></li><li><strong>abcd</strong>ef</li></ol>")
}

#[test]
fn entering_with_entire_selection_in_one_node_deletes_list() {
    let mut model = cm("<ol><li>{abcd}|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<p>&nbsp;|</p>");
}

#[test]
fn entering_with_entire_selection_across_multiple_nodes_deletes_list() {
    let mut model = cm("<ol><li>{abcd</li><li>}|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<p>&nbsp;|</p>");
}

#[test]
fn entering_with_entire_selection_with_formatting() {
    let mut model = cm("<ol><li><b>{abcd}|</b></li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<p>&nbsp;|</p>");
}

#[test]
fn entering_with_subsequent_items() {
    let mut model = cm("<ol><li>abcd|</li><li>ef</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>abcd</li><li>|</li><li>ef</li></ol>")
}

#[test]
fn entering_mid_text_node() {
    let mut model = cm("<ol><li>ab|gh</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>ab</li><li>|gh</li></ol>");
}

#[test]
fn entering_mid_text_node_with_subsequent_items() {
    let mut model = cm("<ol><li>ab|cd</li><li>ef</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>ab</li><li>|cd</li><li>ef</li></ol>")
}

#[test]
fn entering_mid_text_node_with_formatting() {
    let mut model = cm("<ol><li><strong>abc|def</strong></li></ol>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<ol><li><strong>abc</strong></li><li><strong>|def</strong></li></ol>"
    )
}

#[test]
fn entering_mid_text_node_with_multiple_formatting() {
    let mut model = cm("<ol><li><em><strong>abc|def</strong></em></li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li><em><strong>abc</strong></em></li><li><em><strong>|def</strong></em></li></ol>")
}

#[test]
fn entering_mid_text_node_with_leading_formatting() {
    let mut model = cm("<ol><li><strong>abc|d</strong>ef</li></ol>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<ol><li><strong>abc</strong></li><li><strong>|d</strong>ef</li></ol>"
    )
}

#[test]
fn entering_mid_text_node_with_trailing_formatting() {
    let mut model = cm("<ol><li>ab<strong>c|def</strong></li></ol>");
    model.enter();
    assert_eq!(
        tx(&model),
        "<ol><li>ab<strong>c</strong></li><li><strong>|def</strong></li></ol>"
    )
}

#[test]
fn entering_mid_text_node_with_selection() {
    let mut model = cm("<ol><li>ab{cdef}|gh</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>ab</li><li>|gh</li></ol>");
}

#[test]
fn removing_list() {
    let mut model = cm("|");
    model.ordered_list();
    model.enter();
    assert_eq!(tx(&model), "<p>&nbsp;|</p>");
}

#[test]
fn removing_trailing_list_item_with_enter() {
    let mut model = cm("<ol><li>abc</li><li>|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>abc</li></ol><p>&nbsp;|</p>")
}

#[test]
fn removing_trailing_list_item_with_list_toggle() {
    let mut model = cm("<ol><li>abc</li><li>|</li></ol>");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>abc</li></ol><p>&nbsp;|</p>")
}

#[test]
fn removing_trailing_list_item_then_replace_text() {
    let mut model = cm("<ol><li>abc</li><li>|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>abc</li></ol><p>&nbsp;|</p>");
    replace_text(&mut model, "def");
    assert_eq!(tx(&model), "<ol><li>abc</li></ol><p>def|</p>");
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
    assert_eq!(tx(&model), "<ol><li>ab</li></ol><p>cd|</p>");
}

#[test]
fn appending_new_list_to_previous() {
    let mut model = cm("<ol><li>ab</li></ol><p>cd|</p>");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>ab</li><li>cd|</li></ol>");
}

#[test]
fn creating_list_of_different_type_doesnt_merge() {
    let mut model = cm("<ul><li>foo</li></ul>bar|");
    model.ordered_list();
    assert_eq!(tx(&model), "<ul><li>foo</li></ul><ol><li>bar|</li></ol>");
}

#[test]
fn creating_a_new_list_immediately_after_an_old_one_joins_them() {
    let mut model = cm("abc|");
    model.unordered_list();
    model.enter();
    model.enter();
    model.replace_text(Utf16String::from_str("def"));
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>abc</li><li>def|</li></ul>");
}

#[test]
fn indent_single_list_item_works() {
    let mut model = cm(
        "<ul><li>First item</li><li>Second item|</li><li>Third item</li></ul>",
    );
    let (s, e) = model.safe_selection();
    let range = model.state.dom.find_range(s, e);
    let can_indent = model.can_indent(&range.locations);
    assert!(can_indent);
    model.indent();
    assert_eq!(tx(&model), "<ul><li><p>First item</p><ul><li>Second item|</li></ul></li><li>Third item</li></ul>");
}

#[test]
fn indent_single_empty_list_item_works() {
    let mut model =
        cm("<ul><li>First item</li><li>|</li><li>Third item</li></ul>");
    let (s, e) = model.safe_selection();
    let range = model.state.dom.find_range(s, e);
    let can_indent = model.can_indent(&range.locations);
    assert!(can_indent);
    model.indent();
    assert_eq!(tx(&model), "<ul><li><p>First item</p><ul><li>|</li></ul></li><li>Third item</li></ul>");
}

#[test]
fn indent_several_list_items_simple_case_works() {
    let mut model = cm(
        "<ul><li>First item</li><li>{Second item</li><li>Third item}|</li></ul>",
    );
    model.indent();
    assert_eq!(tx(&model), "<ul><li><p>First item</p><ul><li>{Second item</li><li>Third item}|</li></ul></li></ul>");
}

#[test]
fn indent_several_list_items_complex_case_works() {
    let mut model = cm(
        "<ul><li>First item</li><li>{Second item</li><li>Third item</li><li>Fourth item}|</li></ul>",
    );
    model.indent();
    assert_eq!(tx(&model), "<ul><li><p>First item</p><ul><li>{Second item</li><li>Third item</li><li>Fourth item}|</li></ul></li></ul>");
}

#[test]
fn indent_several_list_items_with_sub_levels_works() {
    let mut model = cm(
        "<ul><li><p>First item</p><ul><li>Second item</li><li><p>{Third item</p><ul><li>Fourth item</li><li>Fifth item}|</li></ul></li></ul></li></ul>",
    );
    model.indent();
    assert_eq!(tx(&model), "<ul><li><p>First item</p><ul><li><p>Second item</p><ul><li><p>{Third item</p><ul><li>Fourth item</li><li>Fifth item}|</li></ul></li></ul></li></ul></li></ul>");
}

#[test]
fn unindent_several_items_works() {
    let mut model =
        cm("<ul><li><p>First item</p><ul><li>{Second item</li><li>Third item}|</li></ul></li></ul>");
    model.unindent();
    assert_eq!(
        tx(&model),
        "<ul><li>First item</li><li>{Second item</li><li>Third item}|</li></ul>"
    )
}

#[test]
fn unindent_nested_lists_works() {
    let mut model =
        cm("<ul><li><p>First item</p><ul><li><p>{Second item</p><ul><li>Third item}|</li></ul></li></ul></li></ul>");
    model.unindent();
    assert_eq!(
        tx(&model),
        "<ul><li>First item</li><li><p>{Second item</p><ul><li>Third item}|</li></ul></li></ul>"
    )
}

#[test]
fn unindent_middle_list_item_works() {
    let mut model =
        cm("<ul><li><p>First item</p><ul><li>Second item</li><li>{Third item}|</li><li>Fourth item</li></ul></li></ul>");
    model.unindent();
    assert_eq!(
        tx(&model),
        "<ul><li><p>First item</p><ul><li>Second item</li></ul></li><li><p>{Third item}|</p><ul><li>Fourth item</li></ul></li></ul>"
    )
}

#[test]
fn unindent_nested_lists_with_remnants_works() {
    let mut model =
        cm("<ul><li><p>First item</p><ul><li><p>Second item</p><ul><li>{Third item</li><li>Fourth item}|</li><li>Fifth item</li></ul></li></ul></li></ul>");
    model.unindent();
    assert_eq!(
        tx(&model),
        "<ul><li><p>First item</p><ul><li>Second item</li><li>{Third item</li><li><p>Fourth item}|</p><ul><li>Fifth item</li></ul></li></ul></li></ul>"
    )
}

#[test]
fn replacing_text_with_newline_characters_inserts_list_items() {
    let mut model = cm("<ul><li>abc|</li></ul>");
    replace_text(&mut model, "def\nghi");
    assert_eq!(tx(&model), "<ul><li>abcdef</li><li>ghi|</li></ul>");
}

#[test]
fn replacing_cross_list_item_selection_with_text_containing_newline_works() {
    let mut model = cm("<ul><li>a{bc</li><li>de}|f</li></ul>");
    replace_text(&mut model, "ghi\njkl");
    assert_eq!(tx(&model), "<ul><li>aghi</li><li>jkl|f</li></ul>");
}

#[test]
fn creating_list_from_multiline_selection() {
    let mut model = cm("{abc<br />def}|");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>{abc</li><li>def}|</li></ol>")
}

#[test]
fn creating_list_from_multiline_selection_with_formatting() {
    let mut model = cm("{a<strong>b</strong>c<br />def}|");
    model.ordered_list();
    assert_eq!(
        tx(&model),
        "<ol><li>{a<strong>b</strong>c</li><li>def}|</li></ol>"
    )
}

#[test]
fn creating_list_from_multiline_selection_with_leading_formatting() {
    let mut model = cm("{<strong>ab</strong>c<br />def}|");
    model.ordered_list();
    assert_eq!(
        tx(&model),
        "<ol><li><strong>{ab</strong>c</li><li>def}|</li></ol>"
    )
}

#[test]
fn creating_list_from_multiline_selection_with_trailing_formatting() {
    let mut model = cm("{abc<br />d<strong>ef</strong>}|");
    model.ordered_list();
    assert_eq!(
        tx(&model),
        "<ol><li>{abc</li><li>d<strong>ef}|</strong></li></ol>"
    )
}

#[test]
fn creating_list_from_multiline_selection_with_cross_trailing_formatting() {
    let mut model = cm("{abc<br />d<strong>e}|f</strong>");
    model.ordered_list();
    assert_eq!(
        tx(&model),
        "<ol><li>{abc</li><li>d<strong>e}|f</strong></li></ol>"
    )
}

#[test]
fn creating_list_from_multiline_selection_with_cross_leading_formatting() {
    let mut model = cm("a<em>b{c</em><br />def}|");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>a<em>b{c</em></li><li>def}|</li></ol>")
}

#[test]
fn backspacing_in_empty_list_then_creating_a_new_list() {
    let mut model = cm("<ol><li>|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "|");
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>|</li></ul>");
}

#[test]
fn toggle_list_then_type_text() {
    let mut model = cm("|");
    model.unordered_list();
    model.unordered_list();
    replace_text(&mut model, "A");
    model.backspace();
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>|</li></ul>");
}

#[test]
fn multiple_list_toggle() {
    let mut model = cm("|");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>|</li></ol>");
    model.ordered_list();
    assert_eq!(tx(&model), "<p>&nbsp;|</p>");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>|</li></ol>");
}

#[test]
fn new_list_after_linebreak() {
    let mut model = cm("start|");
    model.enter();
    assert_eq!(tx(&model), "<p>start</p><p>&nbsp;|</p>");

    model.unordered_list();
    // TODO: make 'start' be contained into a paragraph
    assert_eq!(tx(&model), "<p>start</p><ul><li>|</li></ul>");
}

#[test]
fn backspace_text_in_list_item() {
    let mut model = cm("<p>test</p><ol><li>looks good|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "<p>test</p><ol><li>looks goo|</li></ol>")
}

#[test]
fn backspacing_an_empty_indented_list_item_removes_parent_list_item_paragraph()
{
    let mut model = cm("<ul><li><p>item</p><ul><li><p>it</p><ul><li>|</li></ul></li></ul></li></ul>");
    model.backspace();
    assert_eq!(
        tx(&model),
        "<ul><li><p>item</p><ul><li>it|</li></ul></li></ul>"
    );
    model.enter();
    assert_eq!(
        tx(&model),
        "<ul><li><p>item</p><ul><li>it</li><li>|</li></ul></li></ul>"
    );
}

#[test]
fn backspacing_at_a_non_empty_indented_list_item_start_removes_parent_list_item_paragraph(
) {
    let mut model = cm("<ul><li><p>item</p><ul><li><p>it</p><ul><li>|em</li></ul></li></ul></li></ul>");
    model.backspace();
    assert_eq!(
        tx(&model),
        "<ul><li><p>item</p><ul><li>it|em</li></ul></li></ul>"
    );
    model.enter();
    assert_eq!(
        tx(&model),
        "<ul><li><p>item</p><ul><li>it</li><li>|em</li></ul></li></ul>"
    );
}

#[test]
fn backspacing_with_selection_across_an_indented_list_item_removes_parent_list_item_paragraph(
) {
    let mut model = cm("<ul><li><p>item</p><ul><li><p>i{t</p><ul><li>em</li></ul></li></ul></li></ul><p>ab}|c</p>");
    model.backspace();
    assert_eq!(
        tx(&model),
        "<ul><li><p>item</p><ul><li>i|c</li></ul></li></ul>"
    );
    model.enter();
    assert_eq!(
        tx(&model),
        "<ul><li><p>item</p><ul><li>i</li><li>|c</li></ul></li></ul>"
    );
}

#[test]
fn backspacing_an_indented_list_item_with_siblings_doesnt_remove_parent_list_item_paragraph(
) {
    let mut model = cm("<ul><li><p>item</p><ul><li><p>i{t</p><ul><li>em}|</li><li>sibling</li></ul></li></ul></li></ul>");
    model.backspace();
    assert_eq!(
        tx(&model),
        "<ul><li><p>item</p><ul><li><p>i|</p><ul><li>sibling</li></ul></li></ul></li></ul>"
    );
}

fn replace_text(model: &mut ComposerModel<Utf16String>, new_text: &str) {
    model.replace_text(utf16(new_text));
}
