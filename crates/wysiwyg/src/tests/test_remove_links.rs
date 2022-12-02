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

#[test]
fn remove_link_on_a_non_link_node() {
    let mut model = cm("{test}|");
    model.remove_links();
    assert_eq!(tx(&model), "{test}|");
}

#[test]
fn remove_selected_link() {
    let mut model = cm("<a href=\"https://matrix.org\">{test_link}|</a>");
    model.remove_links();
    assert_eq!(tx(&model), "{test_link}|");
}

#[test]
fn remove_selected_link_and_undo() {
    let mut model = cm("<a href=\"https://matrix.org\">{test_link}|</a>");
    model.remove_links();
    assert_eq!(tx(&model), "{test_link}|");
    model.undo();
    assert_eq!(
        tx(&model),
        "<a href=\"https://matrix.org\">{test_link}|</a>"
    );
}

#[test]
fn remove_partially_selected_link() {
    let mut model = cm("<a href=\"https://matrix.org\">{test}|_link</a>");
    model.remove_links();
    assert_eq!(tx(&model), "{test}|_link");
}

#[test]
fn remove_link_in_selected_container() {
    let mut model = cm(
        "<b>{test <a href=\"https://matrix.org\">test_link_bold}|</a></b> test",
    );
    model.remove_links();
    assert_eq!(tx(&model), "<b>{test test_link_bold}|</b> test");
}

#[test]
fn remove_link_that_contains_a_container() {
    let mut model =
        cm("<a href=\"https://matrix.org\"><b>{test_link_bold}|</b></a>");
    model.remove_links();
    assert_eq!(tx(&model), "<b>{test_link_bold}|</b>");
}

#[test]
fn remove_multiple_selected_links() {
    let mut model = cm("<a href=\"https://matrix.org\">{test_link_1</a> <a href=\"https://element.io\">test_link_2}|</a>");
    model.remove_links();
    assert_eq!(tx(&model), "{test_link_1 test_link_2}|");
}

#[test]
fn remove_multiple_partially_selected_links() {
    let mut model = cm("<a href=\"https://matrix.org\">test_{link_1</a> <a href=\"https://element.io\">test}|_link_2</a>");
    model.remove_links();
    assert_eq!(tx(&model), "test_{link_1 test}|_link_2");
}
