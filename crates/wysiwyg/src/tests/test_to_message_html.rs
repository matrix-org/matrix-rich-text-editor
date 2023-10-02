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

use crate::tests::testutils_composer_model::{cm, tx};

#[test]
fn outputs_paragraphs_as_line_breaks() {
    let mut model = cm("|");
    model.replace_text("hello".into());
    model.enter();
    model.enter();
    model.enter();
    model.enter();
    model.replace_text("Alice".into());

    assert_eq!(
        tx(&model),
        "<p>hello</p><p>&nbsp;</p><p>&nbsp;</p><p>&nbsp;</p><p>Alice|</p>"
    );
    let message_output = model.get_content_as_message_html();
    assert_eq!(message_output, "hello<br /><br /><br /><br />Alice");
}

#[test]
fn only_outputs_href_attribute_on_user_mention() {
    let mut model = cm("|");
    model.insert_mention(
        "https://matrix.to/#/@alice:matrix.org".into(),
        "inner text".into(),
        vec![("style".into(), "some css".into())],
    );
    assert_eq!(tx(&model), "<a style=\"some css\" data-mention-type=\"user\" href=\"https://matrix.to/#/@alice:matrix.org\" contenteditable=\"false\">inner text</a>&nbsp;|");

    let message_output = model.get_content_as_message_html();
    assert_eq!(
        message_output,
        "<a href=\"https://matrix.to/#/@alice:matrix.org\">inner text</a>\u{a0}"
    );
}

#[test]
fn only_outputs_href_attribute_on_room_mention_and_uses_mx_id() {
    let mut model = cm("|");
    model.insert_mention(
        "https://matrix.to/#/#alice:matrix.org".into(),
        "inner text".into(),
        vec![("style".into(), "some css".into())],
    );
    assert_eq!(tx(&model), "<a style=\"some css\" data-mention-type=\"room\" href=\"https://matrix.to/#/#alice:matrix.org\" contenteditable=\"false\">inner text</a>&nbsp;|");

    let message_output = model.get_content_as_message_html();
    assert_eq!(
        message_output,
        "<a href=\"https://matrix.to/#/#alice:matrix.org\">#alice:matrix.org</a>\u{a0}"
    );
}

#[test]
fn only_outputs_href_inner_text_for_at_room_mention() {
    let mut model = cm("|");
    model.insert_at_room_mention(vec![("style".into(), "some css".into())]);
    assert_eq!(tx(&model), "<a style=\"some css\" data-mention-type=\"at-room\" href=\"#\" contenteditable=\"false\">@room</a>&nbsp;|");

    let message_output = model.get_content_as_message_html();
    assert_eq!(message_output, "@room\u{a0}");
}
