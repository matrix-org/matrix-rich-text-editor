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
use crate::ToRawText;

#[test]
fn empty_text_converts_to_empty_raw_string() {
    assert_eq!(raw("|"), "");
}

#[test]
fn simple_text_converts_directly_to_raw_version() {
    assert_eq!(raw("abcdef|"), "abcdef");
}

#[test]
fn multi_code_unit_characters_convert_to_raw_text_unchanged() {
    assert_eq!(
        raw("\u{1F469}\u{1F3FF}\u{200D}\u{1F680}|"),
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F680}"
    );
}

#[test]
fn tags_are_stripped_from_raw_text() {
    assert_eq!(raw("t<b>a</b>g|"), "tag",);

    assert_eq!(raw("nes<b>ted<i>tag</i></b>s|"), "nestedtags",);

    assert_eq!(
        raw("some <a href=\"https://matrix.org\">link</a>|"),
        "some link",
    );

    // mention
    assert_eq!(
        raw("some <a href=\"https://matrix.to/#/@test:example.org\">test</a>|"),
        "some test",
    );

    assert_eq!(
        raw("list: <ol><li>ab</li><li>cd</li><li><b>e<i>f</i></b></li></ol>|"),
        "list: abcdef",
    );

    assert_eq!(raw("|emptynodes<b><i></i></b>"), "emptynodes");
}

fn raw(s: &str) -> Utf16String {
    cm(s).state.dom.to_raw_text()
}
