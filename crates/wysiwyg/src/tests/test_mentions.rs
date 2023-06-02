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

use widestring::Utf16String;

use crate::{
    tests::testutils_composer_model::{cm, tx},
    ComposerModel, MenuAction,
};

/**
 * ATTRIBUTE TESTS
 */
#[test]
fn mention_without_attributes() {
    let mut model = cm("|");
    insert_mention_at_cursor(&mut model);

    assert_eq!(
        tx(&model),
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|",
    );
}

#[test]
fn mention_with_attributes() {
    let mut model = cm("|");
    let update = model.replace_text("@alic".into());
    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
    model.insert_mention_at_suggestion(
        "https://matrix.to/#/@alice:matrix.org".into(),
        "Alice".into(),
        suggestion,
        vec![("data-mention-type".into(), "user".into())],
    );
    assert_eq!(
        tx(&model),
        "<a data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|",
    );
}

/**
 * INSERT AT CURSOR
 */
/**
 * TEXT NODE
 */
#[test]
fn text_node_replace_all() {
    let mut model = cm("|");
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|",
    );
}

#[test]
fn text_node_replace_start() {
    let mut model = cm("| says hello");
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| says hello",
    );
}

#[test]
fn text_node_replace_middle() {
    let mut model = cm("Like | said");
    insert_mention_at_cursor(&mut model);
    assert_eq!(tx(&model),
    "Like <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| said");
}

#[test]
fn text_node_replace_end() {
    let mut model = cm("hello |");
    insert_mention_at_cursor(&mut model);
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
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>|<br />",
    );
}

#[test]
fn linebreak_insert_after() {
    let mut model = cm("<br />|");
    insert_mention_at_cursor(&mut model);
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
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>|<a href=\"https://matrix.to/#/@test:example.org\" contenteditable=\"false\">test</a>",
    );
}

#[test]
fn mention_insert_after() {
    let mut model =
        cm("<a href=\"https://matrix.to/#/@test:example.org\" contenteditable=\"false\">test</a>|");
    insert_mention_at_cursor(&mut model);
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
    model.insert_mention_at_suggestion(
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
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<strong><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| says hello</strong>",
    );
}

#[test]
fn formatting_node_replace_middle() {
    let mut model = cm("<strong>Like | said</strong>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<strong>Like <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| said</strong>",
    );
}

#[test]
fn formatting_node_replace_end() {
    let mut model = cm("<strong>hello |</strong>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<strong>hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|</strong>",
    );
}

#[test]
#[should_panic]
fn formatting_node_inline_code() {
    let mut model = cm("<pre>hello |</pre>");
    insert_mention_at_cursor(&mut model);
}

/**
 * LINK NODES
 */
#[test]
fn link_insert_before() {
    let mut model =
        cm("| <a href=\"https://www.somelink.com\">regular link</a>");
    insert_mention_at_cursor(&mut model);
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
    insert_mention_at_cursor(&mut model);
}

#[test]
fn link_insert_after() {
    let mut model =
        cm("<a href=\"https://www.somelink.com\">regular link</a> |");
    insert_mention_at_cursor(&mut model);
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
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<ol><li><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|</li></ol>",
    );
}

#[test]
fn list_item_replace_start() {
    let mut model = cm("<ol><li>| says hello</li></ol>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<ol><li><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| says hello</li></ol>",
    );
}

#[test]
fn list_item_replace_middle() {
    let mut model = cm("<ol><li>Like | said</li></ol>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(tx(&model),
    "<ol><li>Like <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| said</li></ol>");
}

#[test]
fn list_item_replace_end() {
    let mut model = cm("<ol><li>hello |</li></ol>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<ol><li>hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|</li></ol>",
    );
}

/**
 * CodeBlock
 */
#[test]
#[should_panic]
fn codeblock_insert_anywhere() {
    let mut model = cm("<code>regular | link</code>");
    insert_mention_at_cursor(&mut model);
}

/**
 * Quote
 */
#[test]
fn quote_insert_into_empty() {
    let mut model = cm("<blockquote><p>|</p></blockquote>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<blockquote><p><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|</p></blockquote>",
    );
}

#[test]
fn quote_replace_start() {
    let mut model = cm("<blockquote><p>| says hello</p></blockquote>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<blockquote><p><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| says hello</p></blockquote>",
    );
}

#[test]
fn quote_replace_middle() {
    let mut model = cm("<blockquote><p>Like | said</p></blockquote>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(tx(&model),
    "<blockquote><p>Like <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| said</p></blockquote>");
}

#[test]
fn quote_replace_end() {
    let mut model = cm("<blockquote><p>hello |</p></blockquote>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<blockquote><p>hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|</p></blockquote>",
    );
}

/**
 * PARAGRAPH
 */
#[test]
fn paragraph_insert_into_empty() {
    let mut model = cm("<p>|</p>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<p><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|</p>",
    );
}

#[test]
fn paragraph_replace_start() {
    let mut model = cm("<p>| says hello</p>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<p><a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| says hello</p>",
    );
}

#[test]
fn paragraph_replace_middle() {
    let mut model = cm("<p>Like | said</p>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(tx(&model),
    "<p>Like <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>| said</p>");
}

#[test]
fn paragraph_replace_end() {
    let mut model = cm("<p>hello |</p>");
    insert_mention_at_cursor(&mut model);
    assert_eq!(
        tx(&model),
        "<p>hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|</p>",
    );
}

/**
 * INSERT INTO SELECTION
 */

#[test]
fn selection_plain_text_replace() {
    let mut model = cm("{replace_me}|");
    insert_mention_at_selection(&mut model);
    assert_eq!(
        tx(&model),
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>&nbsp;|"
    );
}

#[test]
fn selection_plain_text_start() {
    let mut model = cm("{replace}|_me");
    insert_mention_at_selection(&mut model);
    assert_eq!(
        tx(&model),
        "<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>|_me"
    );
}

#[test]
fn selection_plain_text_middle() {
    let mut model = cm("replac{e}|_me");
    insert_mention_at_selection(&mut model);
    assert_eq!(
        tx(&model),
        "replac<a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>|_me"
    );
}

#[test]
fn selection_formatting_inside() {
    let mut model = cm("<strong>hello {replace_me}|!</strong>");
    insert_mention_at_selection(&mut model);
    assert_eq!(
        tx(&model),
       "<strong>hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>|!</strong>"
    );
}

#[test]
fn selection_formatting_spanning() {
    let mut model = cm("<strong>hello {replace</strong><em>_me}|!</em>");
    insert_mention_at_selection(&mut model);
    assert_eq!(tx(&model), "<strong>hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a></strong><em>&nbsp;|!</em>");
}

#[test]
fn selection_formatting_inline_code() {
    // should not allow insertion
    let mut model = cm("<code>hello {replace_me}|!</code>");
    insert_mention_at_selection(&mut model);
    assert_eq!(tx(&model), "<code>hello {replace_me}|!</code>");
}

// TODO - change behaviour to allow inserting mentions into links
// see issue https://github.com/matrix-org/matrix-rich-text-editor/issues/702
#[test]
fn selection_link_inside() {
    let mut model = cm("<a href=\"something\">hello {replace_me}|!</a>");
    insert_mention_at_selection(&mut model);
    assert_eq!(tx(&model), "<a href=\"something\">hello {replace_me}|!</a>");
}

#[test]
fn selection_link_spanning_partial() {
    let mut model =
        cm("hello {replace<a href=\"something\">_me}|something</a>");
    insert_mention_at_selection(&mut model);
    assert_eq!(
        tx(&model),
        "hello {replace<a href=\"something\">_me}|something</a>"
    );
}

#[test]
fn selection_link_spanning_all() {
    let mut model =
        cm("hello {replace<a href=\"something\">something</a>_me}|!");
    insert_mention_at_selection(&mut model);
    assert_eq!(
        tx(&model),
        "hello {replace<a href=\"something\">something</a>_me}|!"
    );
}

#[test]
fn selection_list_item_spanning() {
    let mut model = cm("<ol><li>hello {replace</li><li>_me}|!</li></ol>");
    insert_mention_at_selection(&mut model);
    assert_eq!(
        tx(&model),
       "<ol><li>hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>|!</li></ol>"
    );
}

#[test]
fn selection_codeblock() {
    // should not allow insertion
    let mut model = cm("<pre><code>hello {replace_me}|!</code></pre>");
    insert_mention_at_selection(&mut model);
    assert_eq!(tx(&model), "<pre><code>hello {replace_me}|!</code></pre>");
}

#[test]
fn selection_quote() {
    let mut model = cm("<blockquote><p>hello {replace_me}|!</p></blockquote>");
    insert_mention_at_selection(&mut model);
    assert_eq!(
        tx(&model),
        "<blockquote><p>hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>|!</p></blockquote>"
    );
}

#[test]
fn selection_paragraph_middle() {
    let mut model = cm("<p>hello {replace_me}|!</p>");
    insert_mention_at_selection(&mut model);
    assert_eq!(
        tx(&model),
        "<p>hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>|!</p>"
    );
}

#[test]
fn selection_paragraph_spanning() {
    let mut model = cm("<p>hello {replace</p><p>_me}|!</p>");
    insert_mention_at_selection(&mut model);
    assert_eq!(
        tx(&model),
        "<p>hello <a href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">Alice</a>|!</p>"
    );
}

/**
 * HELPER FUNCTIONS
 */
fn insert_mention_at_cursor(model: &mut ComposerModel<Utf16String>) {
    let update = model.replace_text("@alic".into());
    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
    model.insert_mention_at_suggestion(
        "https://matrix.to/#/@alice:matrix.org".into(),
        "Alice".into(),
        suggestion,
        vec![],
    );
}

fn insert_mention_at_selection(model: &mut ComposerModel<Utf16String>) {
    model.insert_mention(
        "https://matrix.to/#/@alice:matrix.org".into(),
        "Alice".into(),
        vec![],
    );
}
