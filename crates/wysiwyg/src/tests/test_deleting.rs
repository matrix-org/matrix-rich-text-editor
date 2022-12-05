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

use crate::{
    tests::testutils_composer_model::{cm, restore_whitespace, tx},
    ComposerModel, TextUpdate,
};

#[test]
fn backspacing_a_character_at_the_end_deletes_it() {
    let mut model = cm("abc|");
    model.backspace();
    assert_eq!(tx(&model), "ab|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn backspacing_a_character_at_the_beginning_does_nothing() {
    let mut model = cm("|abc");
    model.backspace();
    assert_eq!(tx(&model), "|abc");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn backspacing_a_character_in_the_middle_deletes_it() {
    let mut model = cm("ab|c");
    model.backspace();
    assert_eq!(tx(&model), "a|c");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn backspacing_a_selection_deletes_it() {
    let mut model = cm("a{bc}|");
    model.backspace();
    assert_eq!(tx(&model), "a|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn backspacing_a_backwards_selection_deletes_it() {
    let mut model = cm("a|{bc}");
    model.backspace();
    assert_eq!(tx(&model), "a|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn backspacing_a_lone_newline_deletes_it() {
    let mut model = ComposerModel::new();
    model.enter();
    model.backspace();
    assert_eq!(tx(&model), "|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn backspacing_a_newline_deletes_it() {
    let mut model = cm("abc|");
    let update = model.enter();

    let replace_all = match update.text_update {
        TextUpdate::Keep => panic!("expected ReplaceAll"),
        TextUpdate::ReplaceAll(replace_all) => replace_all,
        TextUpdate::Select(_) => panic!("expected ReplaceAll"),
    };

    assert_eq!(replace_all.start, 4);
    assert_eq!(replace_all.end, 4);

    model.backspace();
    model.backspace();
    assert_eq!(tx(&model), "ab|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_a_character_at_the_end_does_nothing() {
    let mut model = cm("abc|");
    model.delete();
    assert_eq!(tx(&model), "abc|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_a_character_at_the_beginning_deletes_it() {
    let mut model = cm("|abc");
    model.delete();
    assert_eq!(tx(&model), "|bc");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_a_character_in_the_middle_deletes_it() {
    let mut model = cm("a|bc");
    model.delete();
    assert_eq!(tx(&model), "a|c");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_a_selection_deletes_it() {
    let mut model = cm("a{bc}|");
    model.delete();
    assert_eq!(tx(&model), "a|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_a_backwards_selection_deletes_it() {
    let mut model = cm("a|{bc}");
    model.delete();
    assert_eq!(tx(&model), "a|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_a_range_removes_it() {
    let mut model = cm("abcd|");
    model.delete_in(1, 3);
    assert_eq!(tx(&model), "a|d");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_when_spanning_two_separate_identical_tags_joins_them() {
    let mut model = cm("<b>bo{ld</b> plain <b>BO}|LD</b>");
    model.delete();
    assert_eq!(tx(&model), "<b>bo|LD</b>");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_across_list_items_joins_them() {
    let mut model = cm("<ol>
            <li>1{1</li>
            <li>22</li>
            <li>33</li>
            <li>4}|4</li>
        </ol>");
    model.delete();
    assert_eq!(
        restore_whitespace(&tx(&model)),
        "<ol>
            <li>1|4</li>
        </ol>"
    );
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_across_lists_joins_them() {
    let mut model = cm("<ol>
            <li>1{1</li>
            <li>22</li>
        </ol>
        <ol>
            <li>33</li>
            <li>4}|4</li>
        </ol>");
    model.delete();
    assert_eq!(
        restore_whitespace(&tx(&model)),
        "<ol>
            <li>1|4</li>
        </ol>"
    );
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_across_lists_joins_them_nested() {
    let mut model = cm("<ol>
            <li>1{1</li>
            <li>22</li>
            <ol>
                <li>55</li>
            </ol>
        </ol>
        <ol>
            <li>33</li>
            <li>4}|4</li>
        </ol>");
    model.delete();
    assert_eq!(
        restore_whitespace(&tx(&model)),
        "<ol>
            <li>1|4</li>
        </ol>"
    );
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_across_formatting_different_types() {
    let mut model = cm("<b><i>some {italic</i></b> and}| <b>bold</b> text");
    model.delete();
    assert_eq!(tx(&model), "<b><i>some&nbsp;|</i></b> <b>bold</b> text");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_across_formatting_different_types_on_node_boundary() {
    let mut model = cm("<b><i>some {italic</i></b> and }|<b>bold</b> text");
    model.delete();
    assert_eq!(tx(&model), "<b><i>some&nbsp;|</i>bold</b> text");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_in_nested_structure_and_format_nodes_works() {
    let mut model = cm("<ul><li>A</li><li><b>B{B</b><b>C}|C</b></li></ul>");
    model.delete();
    assert_eq!(tx(&model), "<ul><li>A</li><li><b>B|C</b></li></ul>");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
#[ignore] // TODO: fix this test once this deletion works
fn deleting_empty_list_item() {
    let mut model = cm("<ul><li>A{</li><li>~}|</li></ul>");
    model.backspace();
    assert_eq!(tx(&model), "<ul><li>A|</li></ul>");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_a_newline_deletes_it() {
    let mut model = cm("abc|<br />def");
    model.delete();
    model.delete();
    assert_eq!(tx(&model), "abc|ef");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn test_backspace_emoji() {
    let mut model = cm("😄|😅");
    model.backspace();
    assert_eq!(tx(&model), "|😅");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn test_backspace_complex_emoji() {
    let mut model = cm("Test😮‍💨|😅");
    model.backspace();
    assert_eq!(tx(&model), "Test|😅");
    model.select(6.into(), 6.into());
    model.backspace();
    assert_eq!(tx(&model), "Test|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn test_delete_emoji() {
    let mut model = cm("😄|😅");
    model.delete();
    assert_eq!(tx(&model), "😄|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn test_delete_complex_emoji() {
    let mut model = cm("Test😮‍💨|😅");
    model.delete();
    assert_eq!(tx(&model), "Test😮‍💨|");
    model.select(4.into(), 4.into());
    model.delete();
    assert_eq!(tx(&model), "Test|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn test_delete_complex_grapheme() {
    let mut model = cm("Test|О́");
    model.delete();
    assert_eq!(tx(&model), "Test|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn test_backspace_complex_grapheme() {
    let mut model = cm("TestО́|");
    model.backspace();
    assert_eq!(tx(&model), "Test|");
    model.state.dom.explicitly_assert_invariants();
}

#[test]
fn deleting_initial_text_node_removes_it_completely_without_crashing() {
    let mut model = cm("abc<br />def<br />gh|");
    model.delete_in(4, 10);
    assert_eq!(tx(&model), "abc<br />|",);
}

#[test]
fn deleting_initial_text_node_via_selection_removes_it_completely() {
    let mut model = cm("abc<br />{def<br />gh}|");
    model.delete();
    assert_eq!(tx(&model), "abc<br />|",);
}

#[test]
fn deleting_all_initial_text_and_merging_later_text_produces_one_text_node() {
    let mut model = cm("abc<br />{def<br />gh}|ijk");
    model.delete();
    assert_eq!(tx(&model), "abc<br />|ijk",);
}

#[test]
fn deleting_all_initial_text_within_a_tag_preserves_the_tag() {
    let mut model = cm("abc<br /><strong>{def<br />gh}|ijk</strong>");
    model.delete();
    assert_eq!(tx(&model), "abc<br />|<strong>ijk</strong>",);
}

#[test]
fn deleting_all_text_within_a_tag_deletes_the_tag() {
    let mut model = cm("abc<br /><strong>{def<br />gh}|</strong>ijk");
    model.delete();
    assert_eq!(tx(&model), "abc<br />|ijk",);
}

#[test]
fn deleting_last_character_in_a_container() {
    let mut model = cm("<b>t|</b>");
    model.backspace();
    assert_eq!(tx(&model), "|");
}

#[test]
fn deleting_selection_in_a_container() {
    let mut model = cm("<b>{test}|</b>");
    model.backspace();
    assert_eq!(tx(&model), "|");
}
