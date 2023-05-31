// Copyright 2023 The Matrix.org Foundation C.I.C.
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
    tests::testutils_composer_model::{cm, tx},
    MenuAction,
};

#[test]
fn set_mention_replace_all_text() {
    let mut model = cm("|");
    let update = model.replace_text("@alic".into());
    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
    model.set_mention_from_suggestion(
        "https://matrix.to/#/@alice:matrix.org".into(),
        "Alice".into(),
        suggestion,
        vec![],
    );
    assert_eq!(
        tx(&model),
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|",
    );
}

#[test]
fn set_mention_replace_end_of_text() {
    let mut model = cm("hello |");
    let update = model.replace_text("@ali".into());
    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
    model.set_mention_from_suggestion(
        "https://matrix.to/#/@alice:matrix.org".into(),
        "Alice".into(),
        suggestion,
        vec![],
    );
    assert_eq!(
        tx(&model),
        "hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|",
    );
}

#[test]
fn set_mention_replace_start_of_text() {
    let mut model = cm("| says hello");
    let update = model.replace_text("@ali".into());
    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
    model.set_mention_from_suggestion(
        "https://matrix.to/#/@alice:matrix.org".into(),
        "Alice".into(),
        suggestion,
        vec![],
    );
    assert_eq!(
        tx(&model),
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| says hello",
    );
}

#[test]
#[ignore]
// something weird in tx here - causes a panic thread 'tests::test_mentions::set_mention_replace_middle_of_text' panicked at 'assertion failed: self.is_char_boundary(idx)', /Users/alunturner/.cargo/registry/src/github.com-1ecc6299db9ec823/widestring-1.0.2/src/utfstring.rs:1700:9
/**
* stack backtrace:
  0: rust_begin_unwind
            at /rustc/897e37553bba8b42751c67658967889d11ecd120/library/std/src/panicking.rs:584:5
  1: core::panicking::panic_fmt
            at /rustc/897e37553bba8b42751c67658967889d11ecd120/library/core/src/panicking.rs:142:14
  2: core::panicking::panic
            at /rustc/897e37553bba8b42751c67658967889d11ecd120/library/core/src/panicking.rs:48:5
  3: widestring::utfstring::Utf16String::insert_utfstr
            at /Users/alunturner/.cargo/registry/src/github.com-1ecc6299db9ec823/widestring-1.0.2/src/utfstring.rs:1700:9
  4: <widestring::utfstring::Utf16String as wysiwyg::dom::unicode_string::UnicodeString>::insert
            at ./src/dom/unicode_string.rs:121:9
  5: wysiwyg::composer_model::example_format::SelectionWriter::write_selection_mention_node
            at ./src/composer_model/example_format.rs:371:17
  6: wysiwyg::dom::nodes::mention_node::MentionNode<S>::fmt_mention_html
            at ./src/dom/nodes/mention_node.rs:181:13
  7: <wysiwyg::dom::nodes::mention_node::MentionNode<S> as wysiwyg::dom::to_html::ToHtml<S>>::fmt_html
            at ./src/dom/nodes/mention_node.rs:154:9
  8: <wysiwyg::dom::nodes::dom_node::DomNode<S> as wysiwyg::dom::to_html::ToHtml<S>>::fmt_html
            at ./src/dom/nodes/dom_node.rs:399:36
  9: wysiwyg::dom::nodes::container_node::ContainerNode<S>::fmt_children_html
            at ./src/dom/nodes/container_node.rs:705:17
 10: wysiwyg::dom::nodes::container_node::ContainerNode<S>::fmt_default_html
            at ./src/dom/nodes/container_node.rs:621:9
 11: <wysiwyg::dom::nodes::container_node::ContainerNode<S> as wysiwyg::dom::to_html::ToHtml<S>>::fmt_html
            at ./src/dom/nodes/container_node.rs:603:18
 12: <wysiwyg::dom::nodes::dom_node::DomNode<S> as wysiwyg::dom::to_html::ToHtml<S>>::fmt_html
            at ./src/dom/nodes/dom_node.rs:396:38
 13: wysiwyg::composer_model::example_format::<impl wysiwyg::composer_model::base::ComposerModel<widestring::utfstring::Utf16String>>::to_example_format
            at ./src/composer_model/example_format.rs:278:9
 14: wysiwyg::tests::testutils_composer_model::tx
            at ./src/tests/testutils_composer_model.rs:26:5
 15: wysiwyg::tests::test_mentions::set_mention_replace_middle_of_text
            at ./src/tests/test_mentions.rs:102:16
 16: wysiwyg::tests::test_mentions::set_mention_replace_middle_of_text::{{closure}}
            at ./src/tests/test_mentions.rs:85:1
 17: core::ops::function::FnOnce::call_once
            at /rustc/897e37553bba8b42751c67658967889d11ecd120/library/core/src/ops/function.rs:248:5
 18: core::ops::function::FnOnce::call_once
            at /rustc/897e37553bba8b42751c67658967889d11ecd120/library/core/src/ops/function.rs:248:5
*/
fn set_mention_replace_middle_of_text() {
    let mut model = cm("Like | said");
    let update = model.replace_text("@ali".into());

    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
    model.set_mention_from_suggestion(
        "https://matrix.to/#/@alice:matrix.org".into(),
        "Alice".into(),
        suggestion,
        vec![],
    );
    dbg!(model.get_content_as_html());
    assert_eq!(tx(&model), "Like <a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a> said")
}

#[test]
fn set_mention_replace_all_text_formatting_node() {
    let mut model = cm("<strong>|</strong>");
    let update = model.replace_text("@alic".into());
    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
    model.set_mention_from_suggestion(
        "https://matrix.to/#/@alice:matrix.org".into(),
        "Alice".into(),
        suggestion,
        vec![],
    );
    assert_eq!(
        tx(&model),
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|",
    );
}

#[test]
fn set_mention_replace_end_of_text_formatting_node() {
    let mut model = cm("<strong>hello |</strong>");

    let update = model.replace_text("@ali".into());
    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
    model.set_mention_from_suggestion(
        "https://matrix.to/#/@alice:matrix.org".into(),
        "Alice".into(),
        suggestion,
        vec![],
    );
    assert_eq!(
        tx(&model),
        "<strong>hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|</strong>",
    );
}

#[test]
#[ignore] // same issue here as the big stack trace above

fn set_mention_replace_start_of_text_formatting_node() {
    let mut model = cm("<strong>| says hello</strong>");
    let update = model.replace_text("@ali".into());
    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
    model.set_mention_from_suggestion(
        "https://matrix.to/#/@alice:matrix.org".into(),
        "Alice".into(),
        suggestion,
        vec![],
    );

    dbg!(model.get_content_as_html());
    assert_eq!(
        tx(&model),
        "<strong><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a> says hello</strong>",
    );
}
