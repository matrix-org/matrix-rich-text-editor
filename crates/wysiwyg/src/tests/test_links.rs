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

use crate::tests::testutils_composer_model::cm;

use crate::TextUpdate;

#[test]
fn cant_set_link_to_empty_selection() {
    let mut model = cm("hello |world");
    let update = model.set_link("https://element.io".encode_utf16().collect());
    assert!(matches!(update.text_update, TextUpdate::Keep));
}

#[test]
fn set_link_wraps_selection_in_link_tag() {
    let mut model = cm("{hello}| world");
    model.set_link("https://element.io".encode_utf16().collect());
    assert_eq!(
        model.state.dom.to_string(),
        "<a href=\"https://element.io\">hello</a> world"
    );
}
