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
fn backspacing_a_lone_newline_deletes_it() {
    let mut model = ComposerModel::new();
    model.enter();
    model.backspace();
    assert_eq!(tx(&model), "|");
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
        restore_whitespace(&tx(&model)),
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
        restore_whitespace(&tx(&model)),
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
        restore_whitespace(&tx(&model)),
        "<ol>
            <li>1|4</li>
        </ol>"
    );
}

#[test]
fn deleting_across_formatting_different_types() {
    let mut model = cm("<b><i>some {italic</i></b> and}| <b>bold</b> text");
    model.delete();
    assert_eq!(tx(&model), "<b><i>some&nbsp;|</i></b> <b>bold</b> text");
}

#[test]
fn deleting_across_formatting_different_types_on_node_boundary() {
    let mut model = cm("<b><i>some {italic</i></b> and }|<b>bold</b> text");
    model.delete();
    assert_eq!(tx(&model), "<b><i>some&nbsp;|</i>bold</b> text");
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
    let mut model = cm("<ul><li>A{</li><li>~}|</li></ul>");
    model.backspace();
    assert_eq!(tx(&model), "<ul><li>A|</li></ul>");
}

#[test]
fn deleting_a_newline_deletes_it() {
    let mut model = cm("abc|<br />def");
    model.delete();
    model.delete();
    assert_eq!(tx(&model), "abc|ef");
}

#[test]
fn test_backspace_emoji() {
    let mut model = cm("😄|😅");
    model.backspace();
    assert_eq!(tx(&model), "|😅");
}

#[test]
fn test_backspace_complex_emoji() {
    let mut model = cm("Test😮‍💨|😅");
    model.backspace();
    assert_eq!(tx(&model), "Test|😅");
    model.select(6.into(), 6.into());
    model.backspace();
    assert_eq!(tx(&model), "Test|");
}

#[test]
fn test_delete_emoji() {
    let mut model = cm("😄|😅");
    model.delete();
    assert_eq!(tx(&model), "😄|");
}

#[test]
fn test_delete_complex_emoji() {
    let mut model = cm("Test😮‍💨|😅");
    model.delete();
    assert_eq!(tx(&model), "Test😮‍💨|");
    model.select(4.into(), 4.into());
    model.delete();
    assert_eq!(tx(&model), "Test|");
}

#[test]
fn test_delete_complex_grapheme() {
    let mut model = cm("Test|О́");
    model.delete();
    assert_eq!(tx(&model), "Test|");
}

#[test]
fn test_backspace_complex_grapheme() {
    let mut model = cm("TestО́|");
    model.backspace();
    assert_eq!(tx(&model), "Test|");
}

// Remove word tests, text only. nb these _may_ be considered as superseded by the
// html tests which repeat these exact tests, but wrapped in an <em> tag
#[test]
fn plain_backspace_word_at_beginning_does_nothing() {
    let mut model = cm("|abc");
    model.backspace_word();
    assert_eq!(tx(&model), "|abc")
}
#[test]
fn plain_delete_word_at_end_does_nothing() {
    let mut model = cm("abc|");
    model.delete_word();
    assert_eq!(tx(&model), "abc|")
}

#[test]
fn plain_backspace_word_with_selection_only_removes_selection() {
    let mut model = cm("ab{c def}|");
    model.backspace_word();
    assert_eq!(tx(&model), "ab|")
}
#[test]
fn plain_delete_word_with_selection_only_removes_selection() {
    let mut model = cm("ab{c def}|");
    model.delete_word();
    assert_eq!(tx(&model), "ab|")
}

#[test]
fn plain_backspace_word_at_end_of_single_word_removes_word() {
    let mut model = cm("abc|");
    model.backspace_word();
    assert_eq!(tx(&model), "|")
}
#[test]
fn plain_delete_word_at_start_of_single_word_removes_word() {
    let mut model = cm("|abc");
    model.delete_word();
    assert_eq!(tx(&model), "|")
}

#[test]
fn plain_backspace_word_in_word_removes_start_of_word() {
    let mut model = cm("ab|c");
    model.backspace_word();
    assert_eq!(tx(&model), "|c")
}
#[test]
fn plain_delete_word_in_word_removes_end_of_word() {
    let mut model = cm("a|bc");
    model.delete_word();
    assert_eq!(tx(&model), "a|")
}

#[test]
fn plain_backspace_word_with_multiple_words_removes_single_word() {
    let mut model = cm("abc def| ghi");
    model.backspace_word();
    assert_eq!(restore_whitespace(&tx(&model)), "abc | ghi")
}
#[test]
fn plain_delete_word_with_multiple_words_removes_single_word() {
    let mut model = cm("abc |def ghi");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "abc | ghi")
}

#[test]
fn plain_backspace_word_removes_whitespace_then_word() {
    let mut model = cm("abc def          |");
    model.backspace_word();
    assert_eq!(restore_whitespace(&tx(&model)), "abc |")
}
#[test]
fn plain_delete_word_removes_whitespace_then_word() {
    let mut model = cm("|          abc def");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "| def")
}

#[test]
fn plain_backspace_word_removes_runs_of_non_word_characters() {
    let mut model = cm("abc,.()!@£$^*|");
    model.backspace_word();
    assert_eq!(tx(&model), "abc|")
}
#[test]
fn plain_delete_word_removes_runs_of_non_word_characters() {
    let mut model = cm("|,.()!@£$^*abc");
    model.delete_word();
    assert_eq!(tx(&model), "|abc")
}

#[test]
fn plain_backspace_word_removes_runs_of_non_word_characters_and_whitespace() {
    let mut model = cm("abc  ,.!@£$%       |");
    model.backspace_word();
    assert_eq!(restore_whitespace(&tx(&model)), "abc  |")
}
#[test]
fn plain_delete_word_removes_runs_of_non_word_characters_and_whitespace() {
    let mut model = cm("|  ,.!@£$%  abc");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "|  abc")
}

// Remove word tests including html
#[test]
fn html_backspace_word_at_beginning_does_nothing() {
    let mut model = cm("<em>|abc</em>");
    model.backspace_word();
    assert_eq!(tx(&model), "<em>|abc</em>")
}
#[test]
fn html_delete_word_at_end_does_nothing() {
    let mut model = cm("<em>abc|</em>");
    model.delete_word();
    assert_eq!(tx(&model), "<em>abc|</em>")
}

#[test]
fn html_backspace_word_with_selection_only_removes_selection() {
    let mut model = cm("<em>ab{c def}|</em>");
    model.backspace_word();
    assert_eq!(tx(&model), "<em>ab|</em>")
}
#[test]
fn html_delete_word_with_selection_only_removes_selection() {
    let mut model = cm("<em>ab{c def}|</em>");
    model.delete_word();
    assert_eq!(tx(&model), "<em>ab|</em>")
}

#[test]
fn html_backspace_word_at_end_of_single_word_removes_word() {
    let mut model = cm("<em>abc|</em>");
    model.backspace_word();
    assert_eq!(tx(&model), "<em>|</em>")
}
#[test]
fn html_delete_word_at_start_of_single_word_removes_word() {
    let mut model = cm("<em>|abc</em>");
    model.delete_word();
    assert_eq!(tx(&model), "<em>|</em>")
}

#[test]
fn html_backspace_word_in_word_removes_start_of_word() {
    let mut model = cm("<em>ab|c</em>");
    model.backspace_word();
    assert_eq!(tx(&model), "<em>|c</em>")
}
#[test]
fn html_delete_word_in_word_removes_end_of_word() {
    let mut model = cm("<em>a|bc</em>");
    model.delete_word();
    assert_eq!(tx(&model), "<em>a|</em>")
}

#[test]
fn html_backspace_word_with_multiple_words_removes_single_word() {
    let mut model = cm("<em>abc def| ghi</em>");
    model.backspace_word();
    assert_eq!(restore_whitespace(&tx(&model)), "<em>abc | ghi</em>")
}
#[test]
fn html_delete_word_with_multiple_words_removes_single_word() {
    let mut model = cm("<em>abc |def ghi</em>");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "<em>abc | ghi</em>")
}

#[test]
fn html_backspace_word_removes_whitespace_then_word() {
    let mut model = cm("<em>abc def          |</em>");
    model.backspace_word();
    assert_eq!(restore_whitespace(&tx(&model)), "<em>abc |</em>")
}
#[test]
fn html_delete_word_removes_whitespace_then_word() {
    let mut model = cm("<em>|          abc def</em>");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "<em>| def</em>")
}

#[test]
fn html_backspace_word_removes_runs_of_non_word_characters() {
    let mut model = cm("<em>abc,.()!@£$^*|</em>");
    model.backspace_word();
    assert_eq!(tx(&model), "<em>abc|</em>")
}
#[test]
fn html_delete_word_removes_runs_of_non_word_characters() {
    let mut model = cm("<em>|,.()!@£$^*abc</em>");
    model.delete_word();
    assert_eq!(tx(&model), "<em>|abc</em>")
}

#[test]
fn html_backspace_word_removes_runs_of_non_word_characters_and_whitespace() {
    let mut model = cm("<em>abc  ,.!@£$%       |</em>");
    model.backspace_word();
    assert_eq!(restore_whitespace(&tx(&model)), "<em>abc  |</em>")
}
#[test]
fn html_delete_word_removes_runs_of_non_word_characters_and_whitespace() {
    let mut model = cm("<em>|  ,.!@£$%  abc</em>");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "<em>|  abc</em>")
}

#[test]
fn html_backspace_word_removes_single_linebreak() {
    let mut model = cm("<br />|");
    model.backspace_word();
    assert_eq!(tx(&model), "|")
}
#[test]
fn html_delete_word_removes_single_linebreak() {
    let mut model = cm("|<br />");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "|")
}

#[test]
fn html_backspace_word_removes_only_one_linebreak_of_many() {
    let mut model = cm("<br /><br />|<br />");
    model.backspace_word();
    assert_eq!(tx(&model), "<br />|<br />");
    model.backspace_word();
    assert_eq!(tx(&model), "|<br />")
}
#[test]
fn html_delete_word_removes_only_one_linebreak_of_many() {
    let mut model = cm("<br />|<br /><br />");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "<br />|<br />");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "<br />|")
}

#[test]
fn html_backspace_word_does_not_remove_past_linebreak_in_word() {
    let mut model = cm("a<br />defg|");
    model.backspace_word();
    assert_eq!(tx(&model), "a<br />|")
}
#[test]
fn html_delete_word_does_not_remove_past_linebreak_in_word() {
    let mut model = cm("|abcd<br />f ");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "|<br />f ")
}

#[test]
fn html_backspace_word_at_linebreak_removes_linebreak() {
    let mut model = cm("abc <br/>|");
    model.backspace_word();
    assert_eq!(restore_whitespace(&tx(&model)), "abc |");
}
#[test]
fn html_delete_word_at_linebreak_removes_linebreak() {
    let mut model = cm("|<br/> abc");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "| abc");
}

#[test]
fn html_backspace_word_removes_past_linebreak_in_whitespace() {
    let mut model = cm("abc <br/> |");
    model.backspace_word();
    assert_eq!(restore_whitespace(&tx(&model)), "abc |");
    model.backspace_word();
    assert_eq!(restore_whitespace(&tx(&model)), "|");
}
#[test]
fn html_delete_word_removes_past_linebreak_in_whitespace() {
    let mut model = cm("| <br/> abc");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "| abc");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "|");
}
#[test]
fn html_backspace_word_removes_whole_word() {
    let mut model = cm("<em>italic|</em>");
    model.backspace_word();
    assert_eq!(restore_whitespace(&tx(&model)), "<em>|</em>");
}
#[test]
fn html_delete_word_removes_whole_word() {
    let mut model = cm("<em>|italic</em>");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "<em>|</em>");
}

#[test]
fn html_backspace_word_removes_into_a_tag() {
    let mut model = cm("<em>some em</em>phasis|");
    model.backspace_word();
    assert_eq!(restore_whitespace(&tx(&model)), "<em>some |</em>");
}
#[test]
fn html_delete_word_removes_into_a_tag() {
    let mut model = cm("|so<em>me emphasis</em>");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "|<em> emphasis</em>");
}

#[test]
fn html_backspace_word_removes_through_a_tag() {
    let mut model = cm("si<em>ng</em>le|");
    model.backspace_word();
    assert_eq!(restore_whitespace(&tx(&model)), "|<em></em>");
}
#[test]
fn html_delete_word_removes_through_a_tag() {
    let mut model = cm("|si<em>ng</em>le");
    model.delete_word();
    assert_eq!(restore_whitespace(&tx(&model)), "|<em></em>");
}

#[test]
fn html_backspace_word_removes_between_tags() {
    let mut model = cm("<em>start spl</em><em>it</em>| end");
    model.backspace_word();
    assert_eq!(
        restore_whitespace(&tx(&model)),
        "<em>start |</em><em></em> end"
    );
}
#[test]
fn html_delete_word_removes_between_tags() {
    let mut model = cm("<em>start |spl</em><em>it</em> end");
    model.delete_word();
    assert_eq!(
        restore_whitespace(&tx(&model)),
        "<em>start |</em><em></em> end"
    );
}

// TODO next tests:
// repeat all of the plain text tests inside a tag
// then move on to testing between sibling tags
// then move on to testing with nested tags

// #[test]
// fn backspace_word_multi_step_test() {
//     let mut model = cm(
//         "first   line \n with .,punctuation   \nthird**line \n\n    last  |",
//     );
//     model.backspace_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "first   line \n with .,punctuation   \nthird**line \n\n    |"
//     );
//     model.backspace_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "first   line \n with .,punctuation   \nthird**line \n|"
//     );
//     model.backspace_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "first   line \n with .,punctuation   \nthird**line |"
//     );
//     model.backspace_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "first   line \n with .,punctuation   \nthird**|"
//     );
//     model.backspace_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "first   line \n with .,punctuation   \nthird|"
//     );
//     model.backspace_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "first   line \n with .,punctuation   \n|"
//     );
//     model.backspace_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "first   line \n with .,punctuation   |"
//     );
//     model.backspace_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "first   line \n with .,|");
//     model.backspace_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "first   line \n with |");
//     model.backspace_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "first   line \n |");
//     model.backspace_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "first   line |");
//     model.backspace_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "first   |");
//     model.backspace_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "|")
// }

// #[test]
// fn delete_word_multi_step_test() {
//     let mut model = cm(
//         "|first   line \n with .,punctuation   \nthird**line \n\n    last  ",
//     );
//     model.delete_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "|   line \n with .,punctuation   \nthird**line \n\n    last  "
//     );
//     model.delete_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "| \n with .,punctuation   \nthird**line \n\n    last  "
//     );
//     model.delete_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "| with .,punctuation   \nthird**line \n\n    last  "
//     );
//     model.delete_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "| .,punctuation   \nthird**line \n\n    last  "
//     );
//     model.delete_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "|punctuation   \nthird**line \n\n    last  "
//     );
//     model.delete_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "|   \nthird**line \n\n    last  "
//     );
//     model.delete_word();
//     assert_eq!(
//         restore_whitespace(&tx(&model)),
//         "|third**line \n\n    last  "
//     );
//     model.delete_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "|**line \n\n    last  ");
//     model.delete_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "|line \n\n    last  ");
//     model.delete_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "| \n\n    last  ");
//     model.delete_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "|\n    last  ");
//     model.delete_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "|    last  ");
//     model.delete_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "|  ");
//     model.delete_word();
//     assert_eq!(restore_whitespace(&tx(&model)), "|")
// }
