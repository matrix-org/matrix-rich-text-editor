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

use crate::tests::testutils_composer_model::{cm, tx};
use crate::tests::testutils_conversion::utf16;

use crate::{LinkAction, TextUpdate};

#[test]
fn cant_set_link_to_empty_selection() {
    let mut model = cm("hello |world");
    let update = model.set_link(utf16("https://element.io"));
    assert!(matches!(update.text_update, TextUpdate::Keep));
}

#[test]
fn set_link_wraps_selection_in_link_tag() {
    let mut model = cm("{hello}| world");
    model.set_link(utf16("https://element.io"));
    assert_eq!(
        model.state.dom.to_string(),
        "<a href=\"https://element.io\">hello</a> world"
    );
}

#[test]
fn set_link_in_multiple_leaves_of_formatted_text() {
    let mut model = cm("{<i>test_italic<b>test_italic_bold</b></i>}|");
    model.set_link(utf16("https://element.io"));
    assert_eq!(
        model.state.dom.to_string(),
        "<i><a href=\"https://element.io\">test_italic</a><b><a href=\"https://element.io\">test_italic_bold</a></b></i>"
    )
}

#[test]
fn set_link_in_multiple_leaves_of_formatted_text_partially_covered() {
    let mut model = cm("<i>test_it{alic<b>test_ital}|ic_bold</b></i>");
    model.set_link(utf16("https://element.io"));
    assert_eq!(
        model.state.dom.to_string(),
        "<i>test_it<a href=\"https://element.io\">alic</a><b><a href=\"https://element.io\">test_ital</a>ic_bold</b></i>"
    )
}

#[test]
fn set_link_in_multiple_leaves_of_formatted_text_partially_covered_2() {
    let mut model = cm("<i><u>test_it{alic_underline</u>test_italic<b>test_ital}|ic_bold</b></i>");
    model.set_link(utf16("https://element.io"));
    assert_eq!(
        model.state.dom.to_string(),
        "<i><u>test_it<a href=\"https://element.io\">alic_underline</a></u><a href=\"https://element.io\">test_italic</a><b><a href=\"https://element.io\">test_ital</a>ic_bold</b></i>"
    )
}

#[test]
fn set_link_in_already_linked_text() {
    let mut model = cm("{<a href=\"https://element.io\">link_text</a>}|");
    model.set_link(utf16("https://matrix.org"));
    assert_eq!(
        model.state.dom.to_string(),
        "<a href=\"https://matrix.org\">link_text</a>"
    )
}

#[test]
fn set_link_in_already_linked_text_with_partial_selection() {
    let mut model = cm("<a href=\"https://element.io\">link_{text}|</a>");
    model.set_link(utf16("https://matrix.org"));
    assert_eq!(
        model.state.dom.to_string(),
        "<a href=\"https://matrix.org\">link_text</a>"
    )
}

#[test]
fn set_link_in_text_and_already_linked_text() {
    let mut model =
        cm("{non_link_text<a href=\"https://element.io\">link_text</a>}|");
    model.set_link(utf16("https://matrix.org"));
    assert_eq!(
        model.state.dom.to_string(),
        "<a href=\"https://matrix.org\">non_link_text</a><a href=\"https://matrix.org\">link_text</a>"
    )
}

#[test]
fn set_link_in_multiple_leaves_of_formatted_text_with_link() {
    let mut model = cm("{<i><a href=\"https://element.io\">test_italic</a><b><a href=\"https://element.io\">test_italic_bold</a></b></i>}|");
    model.set_link(utf16("https://matrix.org"));
    assert_eq!(
        model.state.dom.to_string(),
        "<i><a href=\"https://matrix.org\">test_italic</a><b><a href=\"https://matrix.org\">test_italic_bold</a></b></i>"
    )
}

#[test]
fn add_text_at_end_of_link() {
    let mut model = cm("<a href=\"https://element.io\">link|</a>");
    model.replace_text(utf16("added_text"));
    assert_eq!(
        tx(&model),
        "<a href=\"https://element.io\">link</a>added_text|"
    );
}

#[test]
fn add_text_at_end_of_link_inside_a_container() {
    let mut model =
        cm("<b>test <a href=\"https://element.io\">test_link|</a> test</b>");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "<b>test <a href=\"https://element.io\">test_link</a>added_text| test</b>");
}

#[test]
fn replace_text_partially_highlighted_inside_a_link_and_starting_inside() {
    let mut model = cm("<a href=\"https://element.io\">test_{link</a> test}|");
    model.replace_text(utf16("added_text"));
    assert_eq!(
        tx(&model),
        "<a href=\"https://element.io\">test_</a>added_text|"
    );
}

#[test]
fn replace_text_partially_highlighted_inside_a_link_and_starting_inside_in_a_container(
) {
    let mut model =
        cm("<b>test <a href=\"https://element.io\">test_{link</a> test}|</b>");
    model.replace_text(utf16("added_text"));
    assert_eq!(
        tx(&model),
        "<b>test <a href=\"https://element.io\">test_</a>added_text|</b>"
    );
}

#[test]
fn replace_text_partially_highlighted_inside_a_link_and_starting_before() {
    let mut model = cm("{test <a href=\"https://element.io\">test}|_link</a>");
    model.replace_text(utf16("added_text"));
    assert_eq!(
        tx(&model),
        "added_text|<a href=\"https://element.io\">_link</a>"
    );
}

#[test]
fn replace_text_partially_highlighted_inside_a_link_and_starting_before_in_a_container(
) {
    let mut model =
        cm("<b>{test <a href=\"https://element.io\">test}|_link</a> test</b>");
    model.replace_text(utf16("added_text"));
    assert_eq!(
        tx(&model),
        "<b>added_text|<a href=\"https://element.io\">_link</a> test</b>"
    );
}

#[test]
#[ignore]
// TODO: Fix replacing link text when selection starts at the start of the link bug
fn replace_text_with_selection_starting_in_one_link_and_ending_in_another() {
    let mut model =
        cm("test {<a href=\"https://element.io\">test_link_1</a> <a href=\"https://matrix.org\">test_link_2}|</a> test");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "test added_text| test");
}

#[test]
fn replace_text_with_selection_starting_partially_in_one_link_and_ending_in_another_partially(
) {
    let mut model =
        cm("test <a href=\"https://element.io\">test_{link_1</a> <a href=\"https://matrix.org\">test}|_link_2</a> test");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "test <a href=\"https://element.io\">test_</a>added_text|<a href=\"https://matrix.org\">_link_2</a> test");
}

#[test]
#[ignore]
// TODO: Fix replacing link text when selection starts at the start of the link bug
fn replace_text_with_selection_starting_in_one_link_and_ending_in_another_partially(
) {
    let mut model =
        cm("test <a href=\"https://element.io\">{test_link_1</a> <a href=\"https://matrix.org\">test}|_link_2</a> test");
    model.replace_text(utf16("added_text"));
    assert_eq!(
        tx(&model),
        "test added_text|<a href=\"https://matrix.org\">_link_2</a> test"
    );
}

#[test]
fn replace_text_with_selection_starting_partially_in_one_link_and_ending_in_another(
) {
    let mut model =
        cm("test <a href=\"https://element.io\">test_{link_1</a> <a href=\"https://matrix.org\">test_link_2}|</a> test");
    model.replace_text(utf16("added_text"));
    assert_eq!(
        tx(&model),
        "test <a href=\"https://element.io\">test_</a>added_text| test"
    );
}

#[test]
#[ignore]
// TODO: Fix replacing link text when selection starts at the start of the link bug
fn replace_text_over_a_link() {
    let mut model =
        cm("test {<a href=\"https://element.io\">test_link</a>}| test");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "test added_text| test");
}

#[test]
fn replace_text_over_a_link_starting_before() {
    let mut model =
        cm("{test <a href=\"https://element.io\">test_link</a>}| test");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "added_text| test");
}

#[test]
#[ignore]
// TODO: Fix replacing link text when selection starts at the start of the link bug
fn replace_text_over_a_link_ending_after() {
    let mut model =
        cm("test {<a href=\"https://element.io\">test_link</a> test}|");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "test added_text|");
}

#[test]
fn replace_text_over_a_link_starting_before_and_ending_afterwards() {
    let mut model =
        cm("{test <a href=\"https://element.io\">test_link</a> test}|");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "added_text|");
}

#[test]
fn replace_text_in_a_partially_highlighted_container_inside_a_link_at_the_start(
) {
    let mut model =
        cm("<a href=\"https://element.io\"><i><b>{test_bold}|_italic_link</b></i></a>");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "<a href=\"https://element.io\"><i><b>added_text|_italic_link</b></i></a>");
}

#[test]
fn replace_text_in_a_partially_highlighted_container_inside_a_link_starting_and_ending_inside(
) {
    let mut model =
        cm("<a href=\"https://element.io\"><i><b>test_bold_{italic}|_link</b></i></a>");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "<a href=\"https://element.io\"><i><b>test_bold_added_text|_link</b></i></a>");
}

#[test]
fn replace_text_in_a_partially_highlighted_container_inside_a_link_starting_inside_and_ending_at_the_end(
) {
    let mut model =
        cm("<a href=\"https://element.io\"><i><b>test_bold_{italic_link}|</b></i></a>");
    model.replace_text(utf16("added_text"));
    // It looses the bold and italic property, but this is actually google doc's behaviour
    // However we have task to actually support the extension of the contained containers in the future
    // This also only happens when the link is the outermost container
    assert_eq!(tx(&model), "<a href=\"https://element.io\"><i><b>test_bold_</b></i></a>added_text|");
}

#[test]
#[ignore]
// TODO: Fix replacing link text when selection starts at the start of the link bug
fn replace_text_in_a_completely_highlighted_container_inside_a_link() {
    let mut model =
        cm("<a href=\"https://element.io\"><i><b>{test_bold_italic_link}|</b></i></a>");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "added_text|");
}

#[test]
#[ignore]
// TODO: Fix replacing link text when selection starts at the start of the link bug
fn replace_text_in_a_link_inside_a_list() {
    let mut model = cm("<ul><li>list_element</li><li><a href=\"https://element.io\">{link_in_list}|</a></li></ul>");
    model.replace_text(utf16("added_text"));
    assert_eq!(
        tx(&model),
        "<ul><li>list_element</li><li>added_text|</li></ul>"
    );
}

#[test]
fn replace_text_in_a_link_inside_a_list_partially_selected_starting_inside() {
    let mut model = cm("<ul><li>list_element</li><li><a href=\"https://element.io\">link_in_{list}|</a></li></ul>");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "<ul><li>list_element</li><li><a href=\"https://element.io\">link_in_</a>added_text|</li></ul>");
}

#[test]
#[ignore]
// TODO: Fix replacing link text when selection starts at the start of the link bug
fn replace_text_in_a_link_inside_a_list_partially_selected_ending_inside() {
    let mut model = cm("<ul><li>list_element</li><li><a href=\"https://element.io\">{link}|_in_list</a></li></ul>");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "<ul><li>list_element</li><li>added_text|<a href=\"https://element.io\">_in_list</a></li></ul>");
}

#[test]
fn replace_text_in_a_link_inside_a_list_partially_selected_starting_inside_ending_inside(
) {
    let mut model = cm("<ul><li>list_element</li><li><a href=\"https://element.io\">link{_in_}|list</a></li></ul>");
    model.replace_text(utf16("added_text"));
    assert_eq!(tx(&model), "<ul><li>list_element</li><li><a href=\"https://element.io\">linkadded_text|list</a></li></ul>");
}

#[test]
fn link_action_from_non_highlighted_normal_text() {
    let model = cm("test|");
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn link_action_from_highlighted_normal_text() {
    let model = cm("{test}|");
    assert_eq!(model.get_link_action(), LinkAction::Create)
}

#[test]
fn link_action_from_highlighted_link() {
    let model = cm("{<a href=\"https://element.io\">test</a>}|");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit(utf16("https://element.io"))
    )
}
