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

use crate::tests::testutils_composer_model::cm;
use crate::tests::testutils_conversion::utf16;

use crate::{ComposerAction, ComposerModel, Location};

#[test]
fn creating_and_deleting_lists_updates_reversed_actions() {
    let mut model = cm("|");
    model.ordered_list();
    assert!(model.action_is_reversed(ComposerAction::OrderedList));
    assert!(model.action_is_enabled(ComposerAction::UnorderedList));
    model.unordered_list();
    assert!(model.action_is_enabled(ComposerAction::OrderedList));
    assert!(model.action_is_reversed(ComposerAction::UnorderedList));
    model.backspace();
    assert!(model.action_is_enabled(ComposerAction::OrderedList));
    assert!(model.action_is_enabled(ComposerAction::UnorderedList));
}

#[test]
fn selecting_nested_nodes_updates_reversed_actions() {
    let model = cm("<ul><li><b><i>{ab}|</i></b></li></ul>");
    assert!(model.action_is_enabled(ComposerAction::OrderedList));

    assert!(model.action_is_reversed(ComposerAction::UnorderedList));
    assert!(model.action_is_reversed(ComposerAction::Bold));
    assert!(model.action_is_reversed(ComposerAction::Italic));
}

#[test]
fn selecting_multiple_nodes_updates_reversed_actions() {
    let model = cm("<ol><li>{ab</li><li><b>cd</b>}|</li></ol>");
    assert!(model.action_is_reversed(ComposerAction::OrderedList));
    let model = cm("<ol><li>{ab</li></ol>cd}|");
    assert!(model.action_is_enabled(ComposerAction::OrderedList));

    let mut model = cm("<a href=\"https://matrix.org\">{link}|</a>ab");
    assert!(model.action_is_reversed(ComposerAction::Link));
    model.select(Location::from(2), Location::from(6));
    assert!(model.action_is_enabled(ComposerAction::Link));

    let mut model = cm("<del>{ab<em>cd}|</em></del>");
    assert!(model.action_is_reversed(ComposerAction::StrikeThrough));
    model.select(Location::from(2), Location::from(4));
    assert!(model.action_is_reversed(ComposerAction::Italic));
    assert!(model.action_is_reversed(ComposerAction::StrikeThrough));
}

#[test]
fn formatting_updates_reversed_actions() {
    let mut model = cm("a{bc}|d");
    model.bold();
    model.italic();
    model.underline();
    assert!(model.action_is_reversed(ComposerAction::Bold));
    assert!(model.action_is_reversed(ComposerAction::Italic));
    assert!(model.action_is_reversed(ComposerAction::Underline));
}

#[test]
fn updating_model_updates_disabled_actions() {
    let mut model = cm("|");
    assert!(model.action_is_enabled(ComposerAction::Bold));
    assert!(model.action_is_enabled(ComposerAction::Italic));
    assert!(model.action_is_enabled(ComposerAction::StrikeThrough));
    assert!(model.action_is_enabled(ComposerAction::Underline));
    assert!(model.action_is_enabled(ComposerAction::InlineCode));
    assert!(model.action_is_enabled(ComposerAction::Link));
    assert!(model.action_is_enabled(ComposerAction::OrderedList));
    assert!(model.action_is_enabled(ComposerAction::UnorderedList));
    assert!(model.action_is_disabled(ComposerAction::Undo));
    assert!(model.action_is_disabled(ComposerAction::Redo));
    assert!(model.action_is_hidden(ComposerAction::Indent));
    assert!(model.action_is_hidden(ComposerAction::Unindent));

    replace_text(&mut model, "a");
    model.select(Location::from(0), Location::from(1));
    model.bold();
    assert!(model.action_is_disabled(ComposerAction::Redo));
    assert!(model.action_is_hidden(ComposerAction::Indent));
    assert!(model.action_is_hidden(ComposerAction::Unindent));
    assert!(model.action_is_enabled(ComposerAction::StrikeThrough));

    model.undo();
    assert!(model.action_is_enabled(ComposerAction::Redo));

    model.redo();
    assert!(model.action_is_disabled(ComposerAction::Redo));

    model.undo();
    model.undo();
    assert!(model.action_is_disabled(ComposerAction::Undo));
}

#[test]
fn formatting_zero_length_selection_updates_reversed_actions() {
    let mut model = cm("<strong><em>aaa|bbb</em></strong>");
    model.bold();
    model.underline();
    assert!(model.action_is_reversed(ComposerAction::Italic));
    assert!(model.action_is_reversed(ComposerAction::Underline));
    assert!(model.action_is_enabled(ComposerAction::Bold));
}

#[test]
fn selecting_restores_reversed_actions() {
    let mut model = cm("<strong><em>aaa|bbb</em></strong>");
    model.bold();
    model.underline();
    model.select(Location::from(2), Location::from(2));
    assert!(model.action_is_reversed(ComposerAction::Italic));
    assert!(model.action_is_reversed(ComposerAction::Bold));
    assert!(model.action_is_enabled(ComposerAction::StrikeThrough));
}

#[test]
fn test_menu_updates_indent() {
    let model = cm("<ul><li>First item</li><li>{Second item}|</li></ul>");
    assert!(model.action_is_disabled(ComposerAction::Redo));
    assert!(model.action_is_disabled(ComposerAction::Undo));
    assert!(model.action_is_disabled(ComposerAction::Unindent));
    assert!(model.action_is_enabled(ComposerAction::StrikeThrough));
}

#[test]
fn test_menu_updates_unindent() {
    let model =
        cm("<ul><li>First item<ul><li>{Second item}|</li></ul></li></ul>");
    assert!(model.action_is_disabled(ComposerAction::Redo));
    assert!(model.action_is_disabled(ComposerAction::Undo));
    assert!(model.action_is_disabled(ComposerAction::Indent));
    assert!(model.action_is_enabled(ComposerAction::StrikeThrough));
}

#[test]
fn selecting_line_break_inside_formatting_node_reversed_actions() {
    let model = cm("<strong><em>aaa<br />{<br />}|bbb</em></strong>");
    assert!(model.action_is_reversed(ComposerAction::Bold));
    assert!(model.action_is_reversed(ComposerAction::Italic));
    assert!(model.action_is_enabled(ComposerAction::StrikeThrough));
}

#[test]
fn selecting_after_a_line_break_inside_formatting_nodes_reversed_actions() {
    let model = cm("<strong><em>aaa<br /><br />|bbb</em></strong>");
    assert!(model.action_is_reversed(ComposerAction::Bold));
    assert!(model.action_is_reversed(ComposerAction::Italic));
    assert!(model.action_is_enabled(ComposerAction::StrikeThrough));
}

#[test]
fn cursor_at_the_end_of_link_reverses_the_link_action() {
    let model = cm("non_link_text <a href=\"https://element.io\">link_text|</a> non_link_text");
    assert!(model.action_is_reversed(ComposerAction::Link));
}

#[test]
fn formatting_is_disabled_when_selection_is_inside_inline_code_node() {
    let model = cm("<code>Some inl|ine code</code>");
    assert_formatting_actions_and_links_are_disabled(&model);
}

#[test]
fn formatting_is_disabled_when_selection_covers_inline_code_node() {
    let model = cm("<code>{Some inline code}|</code>");
    assert_formatting_actions_and_links_are_disabled(&model);
}

#[test]
fn formatting_is_disabled_when_selection_covers_inline_code_node_and_others() {
    let model =
        cm("<code>{Some inline code</code>, plain text and <b>bold text}|</b>");
    assert_formatting_actions_and_links_are_disabled(&model);
}

#[test]
fn selecting_indented_list_only_marks_the_deepest_list_type_as_reversed() {
    let mut model = cm("<ol><li><p>Item 1</p><ul><li><p>Item 1|A</p><ol><li>Item1A1</li></ol></li></ul></li></ol>");
    assert!(!model.action_is_reversed(ComposerAction::OrderedList));
    assert!(model.action_is_reversed(ComposerAction::UnorderedList));
    // Select inside deeper ordered list.
    model.select(Location::from(15), Location::from(15));
    assert!(!model.action_is_reversed(ComposerAction::UnorderedList));
    assert!(model.action_is_reversed(ComposerAction::OrderedList));
}

#[test]
fn selecting_cross_unmatching_indented_list_exclude_both_list_types() {
    let model =
        cm("<ol><li><p>It{em 1</p><ul><li>Item 1}|A</li></ul></li></ol>");
    assert!(!model.action_is_reversed(ComposerAction::OrderedList));
    assert!(!model.action_is_reversed(ComposerAction::UnorderedList));
}

#[test]
fn selecting_cross_matching_indented_list_include_list_type() {
    let model =
        cm("<ol><li><p>It{em 1</p><ol><li>Item 1}|A</li></ol></li></ol>");
    assert!(model.action_is_reversed(ComposerAction::OrderedList));
}

#[test]
fn clicking_code_block_disables_expected_formatting_functions() {
    let mut model = cm("|");
    model.code_block();
    assert!(model.action_is_disabled(ComposerAction::InlineCode));
    assert!(model.action_is_disabled(ComposerAction::OrderedList));
    assert!(model.action_is_disabled(ComposerAction::UnorderedList));
    assert!(model.action_is_disabled(ComposerAction::Quote));
    assert!(model.action_is_disabled(ComposerAction::Link));
}

#[test]
fn code_block_disables_expected_formatting_functions_with_cursor() {
    let model = cm("<pre><code>Some code| as text</code></pre> and text");
    assert!(model.action_is_disabled(ComposerAction::InlineCode));
    assert!(model.action_is_disabled(ComposerAction::OrderedList));
    assert!(model.action_is_disabled(ComposerAction::UnorderedList));
    assert!(model.action_is_disabled(ComposerAction::Quote));
    assert!(model.action_is_disabled(ComposerAction::Link));
}

#[test]
fn code_block_disables_expected_formatting_functions_with_selection() {
    let model = cm("<pre><code>Some {code as text</code></pre> and}| text");
    assert!(model.action_is_disabled(ComposerAction::InlineCode));
    assert!(model.action_is_disabled(ComposerAction::OrderedList));
    assert!(model.action_is_disabled(ComposerAction::UnorderedList));
    assert!(model.action_is_disabled(ComposerAction::Quote));
    assert!(model.action_is_disabled(ComposerAction::Link));
}

#[test]
fn code_block_doesnt_affect_cursor_if_its_outside() {
    let model = cm("<pre><code>Some code</code></pre><p>|And text</p>");
    assert!(!model.action_is_reversed(ComposerAction::CodeBlock));
    assert!(model.action_is_enabled(ComposerAction::InlineCode));
    assert!(model.action_is_enabled(ComposerAction::Quote));
    assert!(model.action_is_enabled(ComposerAction::UnorderedList));
    assert!(model.action_is_enabled(ComposerAction::OrderedList));
}

#[test]
fn enable_inline_code_with_cursor_immediately_updates_disabled_actions() {
    let mut model = cm("|");
    model.inline_code();
    assert_formatting_actions_and_links_are_disabled(&model);
}

#[test]
fn disable_inline_code_with_cursor_immediately_updates_disabled_actions() {
    let mut model = cm("<code>some code|</code>");
    model.inline_code();
    // Inline code is not marked as reversed anymore
    assert!(!model.action_is_reversed(ComposerAction::InlineCode));
    // Other format types and link are enabled again
    assert!(model.action_is_enabled(ComposerAction::Bold));
    assert!(model.action_is_enabled(ComposerAction::Italic));
    assert!(model.action_is_enabled(ComposerAction::Underline));
    assert!(model.action_is_enabled(ComposerAction::StrikeThrough));
    assert!(model.action_is_enabled(ComposerAction::Link));
}

#[test]
fn empty_paragraph_with_formatting_computes_expected_menu_state() {
    let model = cm("<p><em>abc</em></p><p><em>|</em></p>");
    assert!(model.action_is_reversed(ComposerAction::Italic));
}

#[test]
fn empty_list_item_with_formatting_computes_expected_menu_state() {
    let model = cm("<ol><li><em>abc</em></li><li><em>|</em></li></ol>");
    assert!(model.action_is_reversed(ComposerAction::Italic));
}

#[test]
fn indent_unindent_is_hidden_outside_of_list() {
    let mut model = cm("abc|");
    assert!(model.action_is_hidden(ComposerAction::Indent));
    assert!(model.action_is_hidden(ComposerAction::Unindent));
    model.ordered_list();
    assert!(model.action_is_disabled(ComposerAction::Indent));
    assert!(model.action_is_disabled(ComposerAction::Unindent));
    model.enter();
    assert!(model.action_is_enabled(ComposerAction::Indent));
    assert!(model.action_is_disabled(ComposerAction::Unindent));
}

fn assert_formatting_actions_and_links_are_disabled(
    model: &ComposerModel<Utf16String>,
) {
    assert!(model.action_is_disabled(ComposerAction::Bold));
    assert!(model.action_is_disabled(ComposerAction::Italic));
    assert!(model.action_is_disabled(ComposerAction::Underline));
    assert!(model.action_is_disabled(ComposerAction::StrikeThrough));
    assert!(model.action_is_disabled(ComposerAction::Link));
}

fn replace_text(model: &mut ComposerModel<Utf16String>, new_text: &str) {
    model.replace_text(utf16(new_text));
}
