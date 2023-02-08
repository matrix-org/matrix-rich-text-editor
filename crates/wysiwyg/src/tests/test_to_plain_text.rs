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

use crate::{dom::to_plain_text::ToPlainText, ComposerModel};
use indoc::indoc;
use widestring::Utf16String;

#[test]
fn text() {
    assert_to_plain("abc", "abc");
    assert_to_plain("abc def", "abc def");
    // Internal spaces are preserved.
    assert_to_plain("abc   def", "abc   def");
}

#[test]
fn text_with_linebreaks() {
    // One new line.
    assert_to_plain(
        "abc<br />def",
        indoc! {
            r#"abc
            def"#
        },
    );

    // Two new lines (isn't transformed into a new block).
    assert_to_plain(
        "abc<br /><br />def",
        indoc! {
            r#"abc

            def"#
        },
    );
}

#[test]
fn text_with_italic() {
    assert_to_plain("<em>abc</em>", "abc");
    assert_to_plain("abc <em>def</em> ghi", "abc def ghi");
    assert_to_plain(
        "abc <em>line1<br />line2<br /><br />line3</em> def",
        indoc! {
            r#"abc line1
            line2

            line3 def"#
        },
    );

    assert_to_plain("abc<em>def</em>ghi", "abcdefghi");

    assert_to_plain("abc<em> def </em>ghi", "abc def ghi");
}

#[test]
fn text_with_bold() {
    assert_to_plain("<strong>abc</strong>", "abc");
    assert_to_plain("abc <strong>def</strong> ghi", "abc def ghi");
    assert_to_plain(
        "abc <strong>line1<br />line2<br /><br />line3</strong> def",
        indoc! {
            r#"abc line1
            line2

            line3 def"#
        },
    );

    assert_to_plain("abc<strong>def</strong>ghi", "abcdefghi");

    assert_to_plain("abc<strong> def </strong>ghi", "abc def ghi");
}

#[test]
fn text_with_italic_and_bold() {
    assert_to_plain("<em><strong>abc</strong></em>", "abc");
    assert_to_plain("<em>abc <strong>def</strong></em> ghi", "abc def ghi");
    assert_to_plain(
        "abc <em><strong>line1<br />line2</strong> def</em>",
        indoc! {
            r#"abc line1
            line2 def"#
        },
    );
}

#[test]
fn text_with_strikethrough() {
    assert_to_plain("<del>abc</del>", "abc");
    assert_to_plain("abc <del>def</del> ghi", "abc def ghi");
    assert_to_plain(
        "abc <del>line1<br />line2<br /><br />line3</del> def",
        indoc! {
            r#"abc line1
            line2

            line3 def"#
        },
    );
}

#[test]
fn text_with_underline() {
    assert_to_plain("<u>abc</u>", "abc");
}

#[test]
fn text_with_inline_code() {
    assert_to_plain("<code>abc</code>", "abc");
    // Inline code with a backtick inside.
    assert_to_plain("<code>abc ` def</code>", "abc ` def");
    // Inline code with a backtick at the start.
    assert_to_plain("<code>`abc</code>", "`abc");
    assert_to_plain("abc <code>def</code> ghi", "abc def ghi");
    assert_to_plain("abc<code> def </code>ghi", "abc def ghi");

    assert_to_plain(
        "abc <code>line1<br />line2<br /><br />line3</code> def",
        indoc! {
            r#"abc line1
            line2

            line3 def"#
        },
    );
}

#[test]
fn link() {
    assert_to_plain(r#"<a href="url">abc</a>"#, "abc");
    // Empty link.
    assert_to_plain(r#"<a href="">abc</a>"#, "abc");
    // Formatting inside link.
    assert_to_plain(
        r#"<a href="url">abc <strong>def</strong> ghi</a>"#,
        "abc def ghi",
    );
}

#[test]
fn list_unordered() {
    assert_to_plain(
        r#"<ul><li>item1</li><li>item2</li></ul>"#,
        indoc! {
            r#"item1
            item2
            "#
        },
    );

    assert_to_plain(
        r#"<ul><li>item1<ul><li>subitem1</li><li>subitem2</li></ul></li><li>item2</li></ul>"#,
        indoc! {
            r#"item1
            subitem1
            subitem2
            item2
            "#
        },
    );
}

#[test]
fn list_ordered() {
    assert_to_plain(
        r#"<ol><li>item1</li><li>item2</li></ol>"#,
        indoc! {
            r#"item1
            item2
            "#
        },
    );

    assert_to_plain(
        r#"<ol><li>item1<ol><li>subitem1</li><li>subitem2</li></ol></li><li>item2</li></ol>"#,
        indoc! {
            r#"item1
            subitem1
            subitem2
            item2
            "#
        },
    );
}

#[test]
fn list_ordered_and_unordered() {
    assert_to_plain(
        r#"<ol><li>item1<ul><li>subitem1</li><li>subitem2</li></ul></li><li>item2</li></ol>"#,
        indoc! {
            r#"item1
            subitem1
            subitem2
            item2
            "#
        },
    );
}

#[test]
fn blocks() {
    assert_to_plain(
        r#"<p>paragraph 1</p><ul><li>list item 1</li><li>list item 2</li></ul><pre><code>codeblock</code></pre><blockquote>blockquote</blockquote><p>paragraph 2</p>"#,
        indoc! {
            r#"paragraph 1
            list item 1
            list item 2
            codeblock
            blockquote
            paragraph 2
        "#
        },
    );
}

fn assert_to_plain(html: &str, expected_plain_text: &str) {
    let plain_text = to_plain_text(html);
    assert_eq!(plain_text, expected_plain_text);
}

fn to_plain_text(html: &str) -> Utf16String {
    ComposerModel::from_html(html, 0, 0)
        .state
        .dom
        .to_plain_text()
}
