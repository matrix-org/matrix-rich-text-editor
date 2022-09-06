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

#![cfg(test)]

use crate::tests::testutils_composer_model::{cm, tx};

#[test]
fn backspacing_a_character_at_the_end_deletes_it() {
    let mut model = cm("abc|");
    model.backspace();
    assert_eq!(tx(&model), "ab|");
}

#[test]
fn backspacing_a_character_at_the_beginning_does_nothing() {
    let mut model = cm("|abc");
    model.backspace();
    assert_eq!(tx(&model), "|abc");
}

#[test]
fn backspacing_a_character_in_the_middle_deletes_it() {
    let mut model = cm("ab|c");
    model.backspace();
    assert_eq!(tx(&model), "a|c");
}

#[test]
fn backspacing_a_selection_deletes_it() {
    let mut model = cm("a{bc}|");
    model.backspace();
    assert_eq!(tx(&model), "a|");
}

#[test]
fn backspacing_a_backwards_selection_deletes_it() {
    let mut model = cm("a|{bc}");
    model.backspace();
    assert_eq!(tx(&model), "a|");
}

#[test]
fn deleting_a_character_at_the_end_does_nothing() {
    let mut model = cm("abc|");
    model.delete();
    assert_eq!(tx(&model), "abc|");
}

#[test]
fn deleting_a_character_at_the_beginning_deletes_it() {
    let mut model = cm("|abc");
    model.delete();
    assert_eq!(tx(&model), "|bc");
}

#[test]
fn deleting_a_character_in_the_middle_deletes_it() {
    let mut model = cm("a|bc");
    model.delete();
    assert_eq!(tx(&model), "a|c");
}

#[test]
fn deleting_a_selection_deletes_it() {
    let mut model = cm("a{bc}|");
    model.delete();
    assert_eq!(tx(&model), "a|");
}

#[test]
fn deleting_a_backwards_selection_deletes_it() {
    let mut model = cm("a|{bc}");
    model.delete();
    assert_eq!(tx(&model), "a|");
}

#[test]
fn deleting_a_range_removes_it() {
    let mut model = cm("abcd|");
    model.delete_in(1, 3);
    assert_eq!(tx(&model), "a|d");
}

#[test]
fn deleting_when_spanning_two_separate_identical_tags_joins_them() {
    let mut model = cm("<b>bo{ld</b> plain <b>BO}|LD</b>");
    model.delete();
    assert_eq!(tx(&model), "<b>bo|LD</b>");
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
        tx(&model),
        "<ol>
            <li>1|4</li>
        </ol>"
    );
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
        tx(&model),
        "<ol>
            <li>1|4</li>
        </ol>"
    );
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
        tx(&model),
        "<ol>
            <li>1|4</li>
        </ol>"
    );
}

#[test]
fn deleting_across_formatting_different_types() {
    let mut model = cm("<b><i>some {italic</i></b> and}| <b>bold</b> text");
    model.delete();
    assert_eq!(tx(&model), "<b><i>some |</i></b> <b>bold</b> text");
}

#[test]
fn deleting_across_formatting_different_types_on_node_boundary() {
    let mut model = cm("<b><i>some {italic</i></b> and }|<b>bold</b> text");
    model.delete();
    assert_eq!(tx(&model), "<b><i>some |</i>bold</b> text");
}

#[test]
fn deleting_in_nested_structure_and_format_nodes_works() {
    let mut model = cm("<ul><li>A</li><li><b>B{B</b><b>C}|C</b></li></ul>");
    model.delete();
    assert_eq!(tx(&model), "<ul><li>A</li><li><b>B|C</b></li></ul>");
}

#[test]
#[ignore] // TODO: fix this test once this deletion works
fn deleting_empty_list_item() {
    let mut model = cm("<ul><li>A{</li><li>\u{200B}}|</li></ul>");
    model.backspace();
    assert_eq!(tx(&model), "<ul><li>A|</li></ul>");
}
