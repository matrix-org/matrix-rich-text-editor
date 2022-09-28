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

use crate::{tests::testutils_composer_model::cm, ToMarkdown};
use widestring::Utf16String;

#[test]
fn text() {
    assert_eq!(md("abc|"), "abc");
}

#[test]
fn text_with_linebreaks() {
    // One new line.
    assert_eq!(
        md("abc<br />def|"),
        r#"abc\
def"#
    );

    // Two new lines (isn't transformed into a new block).
    assert_eq!(
        md("abc<br /><br />def|"),
        r#"abc\
\
def"#,
    );
}

#[test]
fn test_with_italic() {
    assert_eq!(md("<em>abc</em>|"), "*abc*");
    // Internal emphasis.
    assert_eq!(md("abc<em>def</em>ghi|"), "abc*def*ghi");
    assert_eq!(md("abc <em>def</em> ghi|"), "abc *def* ghi");
    assert_eq!(md("abc<em> def </em>ghi|"), "abc* def *ghi");
    assert_eq!(
        md("abc <em>line1<br />line2<br /><br />line3</em> def|"),
        r#"abc *line1\
line2\
\
line3* def"#,
    );
}

#[test]
fn test_with_bold() {
    assert_eq!(md("<strong>abc</strong>|"), "__abc__");
    assert_eq!(md("abc<strong>def</strong>ghi|"), "abc__def__ghi");
    assert_eq!(md("abc <strong>def</strong> ghi|"), "abc __def__ ghi");
    assert_eq!(md("abc<strong> def </strong>ghi|"), "abc__ def __ghi");
    assert_eq!(
        md("abc <strong>line1<br />line2<br /><br />line3</strong> def|"),
        r#"abc __line1\
line2\
\
line3__ def"#,
    );
}

#[test]
fn test_with_italic_and_bold() {
    assert_eq!(md("<em><strong>abc</strong></em>|"), "*__abc__*");
    assert_eq!(
        md("<em>abc <strong>def</strong></em> ghi|"),
        "*abc __def__* ghi"
    );
    assert_eq!(
        md("abc <em><strong>line1<br />line2</strong> def</em>|"),
        r#"abc *__line1\
line2__ def*"#,
    );
}

fn md(html: &str) -> Utf16String {
    cm(html).state.dom.to_markdown().unwrap()
}
