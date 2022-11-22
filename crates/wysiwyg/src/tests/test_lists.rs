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
    assert_eq!(tx(&model), "<ol><li>~|</li></ol>");
    replace_text(&mut model, "abcd");
    assert_eq!(tx(&model), "<ol><li>~abcd|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>~abcd</li><li>~|</li></ol>");
    replace_text(&mut model, "efgh");
    assert_eq!(tx(&model), "<ol><li>~abcd</li><li>~efgh|</li></ol>");
}

#[test]
fn creating_unordered_list() {
    let mut model = cm("|");
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>~|</li></ul>");
}

#[test]
#[ignore] // TODO: empty model probably as no current selection to update, so the '|' is misplaced
fn can_create_list_in_empty_model() {
    let mut model = ComposerModel::new();
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>~|</li></ul>");
}

#[test]
fn removing_list_item() {
    let mut model = cm("<ol><li>~abcd</li><li>~|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>~abcd</li></ol>~|");

    let mut model = cm("<ol><li>~abcd</li><li>~|</li></ol>");
    model.backspace();
    assert_eq!(tx(&model), "<ol><li>~abcd|</li></ol>");

    let mut model = cm("<ol><li>~|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "|");

    let mut model = cm("<ol><li>~|</li></ol>");
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
fn entering_with_entire_selection_in_one_node_deletes_list() {
    let mut model = cm("<ol><li>~{abcd}|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "|");
}

#[test]
fn entering_with_entire_selection_across_multiple_nodes_deletes_list() {
    let mut model = cm("<ol><li>~{abcd</li><li>~}|</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "|");
}

#[test]
fn entering_with_entire_selection_with_formatting() {
    let mut model = cm("<ol><li><b>~{abcd}|</b></li></ol>");
    model.enter();
    assert_eq!(tx(&model), "|");
}

#[test]
fn entering_mid_text_node() {
    let mut model = cm("<ol><li>~ab|gh</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>~ab</li><li>~|gh</li></ol>");
}

#[test]
fn entering_mid_text_node_with_selection() {
    let mut model = cm("<ol><li>~ab{cdef}|gh</li></ol>");
    model.enter();
    assert_eq!(tx(&model), "<ol><li>~ab</li><li>~|gh</li></ol>");
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
    let mut model = cm("<ol><li>~ab</li><li>~cd|</li></ol>");
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>~ab</li><li>~cd|</li></ul>");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>~ab</li><li>~cd|</li></ol>");
}

#[test]
#[ignore] // TODO: should remove starting ZWSP when moving out of list
fn moving_list_item_content_out() {
    let mut model = cm("<ol><li>~ab</li><li>~cd|</li></ol>");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>~ab</li></ol>cd|");
}

#[test]
#[ignore] // TODO: selection did not move in that case
fn appending_new_list_to_previous() {
    let mut model = cm("<ol><li>~ab</li></ol>cd|");
    model.ordered_list();
    assert_eq!(tx(&model), "<ol><li>~ab</li><li>~cd|</li></ol>");
}

#[test]
fn creating_list_of_different_type_doesnt_merge() {
    let mut model = cm("<ul><li>~foo</li></ul>bar|");
    model.ordered_list();
    assert_eq!(tx(&model), "<ul><li>~foo</li></ul><ol><li>~bar|</li></ol>");
}

#[test]
fn creating_a_new_list_immediately_after_an_old_one_joins_them() {
    let mut model = cm("abc|");
    model.unordered_list();
    model.enter();
    model.enter();
    model.replace_text(Utf16String::from_str("def"));
    model.unordered_list();
    assert_eq!(tx(&model), "<ul><li>~abc</li><li>~def|</li></ul>");
}

#[test]
fn indent_several_list_items_simple_case_works() {
    let mut model = cm(
        "<ul><li>~First item</li><li>~{Second item</li><li>~Third item}|</li></ul>",
    );
    model.indent();
    assert_eq!(tx(&model), "<ul><li>~First item<ul><li>~{Second item</li><li>~Third item}|</li></ul></li></ul>");
}

#[test]
fn indent_several_list_items_complex_case_works() {
    let mut model = cm(
        "<ul><li>~First item</li><li>~{Second item</li><li>~Third item</li><li>~Fourth item}|</li></ul>",
    );
    model.indent();
    assert_eq!(tx(&model), "<ul><li>~First item<ul><li>~{Second item</li><li>~Third item</li><li>~Fourth item}|</li></ul></li></ul>");
}

#[test]
fn indent_several_list_items_with_sub_levels_works() {
    let mut model = cm(
        "<ul><li>~First item<ul><li>~Second item<ul><li>~Third item</li><li>~{Fourth item</li></ul></li><li>~Fifth item}|</li></ul></li></ul>",
    );
    model.indent();
    assert_eq!(tx(&model), "<ul><li>~First item<ul><li>~Second item<ul><li>~Third item<ul><li>~{Fourth item</li></ul></li><li>~Fifth item}|</li></ul></li></ul></li></ul>");
}

#[test]
fn un_indent_several_items_works() {
    let mut model =
        cm("<ul><li>~First item<ul><li>~{Second item</li><li>~Third item}|</li></ul></li></ul>");
    model.unindent();
    assert_eq!(
        tx(&model),
        "<ul><li>~First item</li><li>~{Second item</li><li>~Third item}|</li></ul>"
    )
}

#[test]
fn un_indent_nested_lists_works() {
    let mut model =
        cm("<ul><li>~First item<ul><li>~{Second item<ul><li>~Third item}|</li></ul></li></ul></li></ul>");
    model.unindent();
    assert_eq!(
        tx(&model),
        "<ul><li>~First item</li><li>~{Second item<ul><li>~Third item}|</li></ul></li></ul>"
    )
}

#[test]
fn un_indent_nested_lists_with_remnants_works() {
    let mut model =
        cm("<ul><li>First item<ul><li>{Second item<ul><li>Third item</li><li>Fourth item}|</li><li>Fifth item</li></ul></li></ul></li></ul>");
    model.unindent();
    assert_eq!(
        tx(&model),
        "<ul><li>First item</li><li>{Second item<ul><li>Third item</li><li>Fourth item}|<ul><li>Fifth item</li></ul></li></ul></li></ul>"
    )
}

#[test]
fn replacing_text_with_newline_characters_inserts_list_items() {
    let mut model = cm("<ul><li>~abc|</li></ul>");
    replace_text(&mut model, "def\nghi");
    assert_eq!(tx(&model), "<ul><li>~abcdef</li><li>~ghi|</li></ul>");
}

#[test]
fn replacing_selection_containing_zwsp_works() {
    let mut model = cm("<ul><li>~a{bc</li><li>~de}|f</li></ul>");
    replace_text(&mut model, "ghi");
    assert_eq!(tx(&model), "<ul><li>~aghi|f</li></ul>");
}

#[test]
fn replacing_selection_containing_zwsp_with_text_containing_trailing_newline_works() {
    let mut model = cm("<ul><li>~a{bc</li><li>~de}|f</li></ul>");
    replace_text(&mut model, "ghi\n");
    assert_eq!(tx(&model), "<ul><li>~aghi</li><li>~|f</li></ul>");
}

#[test]
fn replacing_cross_list_item_selection_with_text_containing_newline_works() {
    let mut model = cm("<ul><li>~a{bc</li><li>~de}|f</li></ul>");
    replace_text(&mut model, "ghi\njkl");
    assert_eq!(tx(&model), "<ul><li>~aghi</li><li>~jkl|f</li></ul>");
}

fn replace_text(model: &mut ComposerModel<Utf16String>, new_text: &str) {
    model.replace_text(utf16(new_text));
}
