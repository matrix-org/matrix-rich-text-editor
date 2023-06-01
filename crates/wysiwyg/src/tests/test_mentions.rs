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

/**
 * TEXT NODE
 */
#[test]
fn text_node_replace_all() {
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
fn text_node_replace_start() {
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
fn text_node_replace_middle() {
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
    assert_eq!(tx(&model),
    "Like <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| said");
}

#[test]
fn text_node_replace_end() {
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

/**
 * LINEBREAK NODES
 */
#[test]
fn linebreak_insert_before() {
    let mut model = cm("|<br />");
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
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>|<br />",
    );
}

#[test]
fn linebreak_insert_after() {
    let mut model = cm("<br />|");
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
        "<br /><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|",
    );
}

/**
 * MENTION NODES
 */
#[test]
fn mention_insert_before() {
    let mut model = cm("|<a href=\"https://matrix.to/#/@test:example.org\" contenteditable=\"false\">test</a>");
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
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>|<a href=\"https://matrix.to/#/@test:example.org\" contenteditable=\"false\">test</a>",
    );
}

#[test]
fn mention_insert_after() {
    let mut model =
        cm("<a href=\"https://matrix.to/#/@test:example.org\" contenteditable=\"false\">test</a>|");
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
        "<a href=\"https://matrix.to/#/@test:example.org\" contenteditable=\"false\">test</a><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|",
    );
}

/**
 * CONTAINER NODES
 */

/**
 * FORMATTING NODES
 */
#[test]
fn formatting_node_replace_all() {
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
fn formatting_node_replace_start() {
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

    assert_eq!(
        tx(&model),
        "<strong><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| says hello</strong>",
    );
}

#[test]
fn formatting_node_replace_middle() {
    let mut model = cm("<strong>Like | said</strong>");
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
        "<strong>Like <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| said</strong>",
    );
}

#[test]
fn formatting_node_replace_end() {
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

/**
 * LINK NODES
 */
#[test]
fn link_insert_before() {
    let mut model =
        cm("| <a href=\"https://www.somelink.com\">regular link</a>");
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
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| <a href=\"https://www.somelink.com\">regular link</a>",
    );
}

// TODO - change behaviour to allow inserting mentions into links
// see issue https://github.com/matrix-org/matrix-rich-text-editor/issues/702
#[test]
#[should_panic]
fn link_insert_middle() {
    let mut model =
        cm("<a href=\"https://www.somelink.com\">regular | link</a>");
    let update = model.replace_text("@ali".into());
    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
}

#[test]
fn link_insert_after() {
    let mut model =
        cm("<a href=\"https://www.somelink.com\">regular link|</a>");
    let update = model.replace_text(" @ali".into());
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
        "<a href=\"https://www.somelink.com\">regular link</a> <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|",
    );
}

/**
 * LIST ITEM
 */
#[test]
fn list_item_insert_into_empty() {
    let mut model = cm("<ol><li>|</li></ol>");
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
        "<ol><li><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|</li></ol>",
    );
}

#[test]
fn list_item_replace_start() {
    let mut model = cm("<ol><li>| says hello</li></ol>");
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
        "<ol><li><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| says hello</li></ol>",
    );
}

#[test]
fn list_item_replace_middle() {
    let mut model = cm("<ol><li>Like | said</li></ol>");
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
    assert_eq!(tx(&model),
    "<ol><li>Like <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| said</li></ol>");
}

#[test]
fn list_item_replace_end() {
    let mut model = cm("<ol><li>hello |</li></ol>");
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
        "<ol><li>hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|</li></ol>",
    );
}
