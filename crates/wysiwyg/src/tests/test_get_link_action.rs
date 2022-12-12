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

use crate::tests::testutils_composer_model::cm;
use crate::tests::testutils_conversion::utf16;

use crate::LinkAction;

#[test]
fn get_link_action_from_cursor_at_end_of_normal_text() {
    let model = cm("test|");
    assert_eq!(model.get_link_action(), LinkAction::Insert)
}

#[test]
fn get_link_action_from_highlighted_normal_text() {
    let model = cm("{test}|");
    assert_eq!(model.get_link_action(), LinkAction::Create)
}

#[test]
fn get_link_action_from_cursor_inside_a_container() {
    let model = cm("<b><i> test_bold_italic |</i> test_bold </b>");
    assert_eq!(model.get_link_action(), LinkAction::Insert)
}

#[test]
fn get_link_action_from_cursor_inside_text() {
    let model = cm("<b><i> test_bold|_italic </i> test_bold </b>");
    assert_eq!(model.get_link_action(), LinkAction::Insert)
}

#[test]
fn get_link_action_from_selection_inside_a_container() {
    let model = cm("<b><i> {test_bold_italic </i> test}|_bold </b>");
    assert_eq!(model.get_link_action(), LinkAction::Create)
}

#[test]
fn get_link_action_from_highlighted_link() {
    let model = cm("{<a href=\"https://element.io\">test</a>}|");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit {
            link: utf16("https://element.io"),
            text: utf16("test")
        }
    )
}

#[test]
fn get_link_action_from_cursor_at_the_end_of_a_link() {
    let model = cm("<a href=\"https://element.io\">test</a>|");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit {
            link: utf16("https://element.io"),
            text: utf16("test")
        }
    )
}

#[test]
fn get_link_action_from_cursor_inside_a_link() {
    let model = cm("<a href=\"https://element.io\">te|st</a>");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit {
            link: utf16("https://element.io"),
            text: utf16("test")
        }
    )
}

#[test]
fn get_link_action_from_cursor_at_the_start_of_a_link() {
    let model = cm("|<a href=\"https://element.io\">test</a>");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit {
            link: utf16("https://element.io"),
            text: utf16("test")
        }
    )
}

#[test]
fn get_link_action_from_selection_that_contains_a_link_and_non_links() {
    let model = cm("<b>{test_bold <a href=\"https://element.io\">test}|_link</a> test_bold</b>");
    assert_eq!(model.get_link_action(), LinkAction::Create)
}

#[test]
fn get_link_action_from_selection_that_contains_multiple_links() {
    let model = cm("{<a href=\"https://element.io\">test_element</a> <a href=\"https://matrix.org\">test_matrix</a>}|");
    assert_eq!(model.get_link_action(), LinkAction::Create)
}

#[test]
fn get_link_action_from_selection_that_contains_multiple_links_partially() {
    let model = cm("<a href=\"https://element.io\">test_{element</a> <a href=\"https://matrix.org\">test}|_matrix</a>");
    assert_eq!(model.get_link_action(), LinkAction::Create)
}

#[test]
fn get_link_action_from_selection_that_contains_multiple_links_partially_in_different_containers(
) {
    let model = cm("<a href=\"https://element.io\"> <b>test_{element</b></a> <i><a href=\"https://matrix.org\">test}|_matrix</a></i>");
    assert_eq!(model.get_link_action(), LinkAction::Create)
}

#[test]
fn get_link_action_returns_edit_when_whole_link_is_selected() {
    let model = cm("{<a href=\"https://element.io\">link_text</a>}|");
    let action = model.get_link_action();
    assert_eq!(
        action,
        LinkAction::Edit {
            link: utf16("https://element.io"),
            text: utf16("link_text"),
        }
    )
}

#[test]
fn get_link_action_returns_edit_when_part_link_is_selected() {
    let model = cm("<a href=\"https://element.io\">link_{text</a>}|");
    let action = model.get_link_action();
    assert_eq!(
        action,
        LinkAction::Edit {
            link: utf16("https://element.io"),
            text: utf16("link_text"),
        }
    )
}

#[test]
fn get_link_action_returns_edit_when_cursor_is_on_link() {
    let model = cm("<a href=\"https://element.io\">link_|text</a>");
    let action = model.get_link_action();
    assert_eq!(
        action,
        LinkAction::Edit {
            link: utf16("https://element.io"),
            text: utf16("link_text"),
        }
    )
}

#[test]
fn get_link_action_returns_create_when_more_than_link_is_selected() {
    let model = cm("{non_link <a href=\"https://element.io\">link_text</a>}|");
    let action = model.get_link_action();
    assert_eq!(action, LinkAction::Create)
}

#[test]
fn get_link_action_returns_create_with_text_when_cursor_is_outside_link() {
    let model = cm("non_link| <a href=\"https://element.io\">link_text</a>");
    let action = model.get_link_action();
    assert_eq!(action, LinkAction::Insert)
}
