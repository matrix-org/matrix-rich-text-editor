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
use crate::ToRawText;

#[test]
fn computing_raw_text() {
    assert_eq!(cm("|").state.dom.to_raw_text(), "",);
    assert_eq!(cm("abcdef|").state.dom.to_raw_text(), "abcdef",);
    assert_eq!(
        cm("\u{1F469}\u{1F3FF}\u{200D}\u{1F680}|")
            .state
            .dom
            .to_raw_text(),
        "\u{1F469}\u{1F3FF}\u{200D}\u{1F680}"
    );
    assert_eq!(cm("t<b>a</b>g|").state.dom.to_raw_text(), "tag",);
    assert_eq!(
        cm("nes<b>ted<i>tag</i></b>s|").state.dom.to_raw_text(),
        "nestedtags",
    );
    assert_eq!(
        cm("some <a href=\"https://matrix.org\">link</a>|")
            .state
            .dom
            .to_raw_text(),
        "some link",
    );
    assert_eq!(
        cm("list: <ol><li>ab</li><li>cd</li><li><b>e<i>f</i></b></li></ol>|")
            .state
            .dom
            .to_raw_text(),
        "list: abcdef",
    );
    assert_eq!(
        cm("emptynodes<b><i></i></b>|").state.dom.to_raw_text(),
        "emptynodes",
    );
}
