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
    assert_eq!(model.get_link_action(), LinkAction::CreateWithText)
}

#[test]
fn get_link_action_from_highlighted_normal_text() {
    let model = cm("{test}|");
    assert_eq!(model.get_link_action(), LinkAction::Create)
}

#[test]
fn get_link_action_from_highlighted_link() {
    let model = cm("{<a href=\"https://element.io\">test</a>}|");
    assert_eq!(
        model.get_link_action(),
        LinkAction::Edit(utf16("https://element.io"))
    )
}
