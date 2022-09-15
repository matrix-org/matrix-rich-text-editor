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

use std::collections::HashSet;

use widestring::Utf16String;

use crate::tests::testutils_composer_model::cm;
use crate::tests::testutils_conversion::utf16;

use crate::{ComposerAction, ComposerModel, InlineFormatType, Location};

#[test]
fn creating_and_deleting_lists_updates_reversed_actions() {
    let mut model = cm("|");
    model.ordered_list();
    assert_eq!(
        model.reversed_actions,
        HashSet::from([ComposerAction::OrderedList])
    );
    model.unordered_list();
    assert_eq!(
        model.reversed_actions,
        HashSet::from([ComposerAction::UnorderedList])
    );
    model.backspace();
    assert_eq!(model.reversed_actions, HashSet::new());
}

#[test]
fn selecting_nested_nodes_updates_reversed_actions() {
    let model = cm("<ul><li><b><i>{ab}|</i></b></li></ul>");
    assert_eq!(
        model.reversed_actions,
        HashSet::from([
            ComposerAction::UnorderedList,
            ComposerAction::Bold,
            ComposerAction::Italic,
        ]),
    );
}

#[test]
fn selecting_multiple_nodes_updates_reversed_actions() {
    let model = cm("<ol><li>{ab</li><li><b>cd</b>}|</li></ol>");
    assert_eq!(
        model.reversed_actions,
        HashSet::from([ComposerAction::OrderedList])
    );
    let model = cm("<ol><li>{ab</li></ol>cd}|");
    assert_eq!(model.reversed_actions, HashSet::new());

    let mut model = cm("<a href=\"https://matrix.org\">{link}|</a>ab");
    assert_eq!(
        model.reversed_actions,
        HashSet::from([ComposerAction::Link])
    );
    model.select(Location::from(2), Location::from(6));
    assert_eq!(model.reversed_actions, HashSet::new());

    let mut model = cm("<del>{ab<em>cd}|</em></del>");
    assert_eq!(
        model.reversed_actions,
        HashSet::from([ComposerAction::StrikeThrough]),
    );
    model.select(Location::from(2), Location::from(4));
    assert_eq!(
        model.reversed_actions,
        HashSet::from([ComposerAction::Italic, ComposerAction::StrikeThrough,]),
    )
}

#[test]
fn formatting_updates_reversed_actions() {
    let mut model = cm("a{bc}|d");
    model.format(InlineFormatType::Bold);
    model.format(InlineFormatType::Italic);
    model.format(InlineFormatType::Underline);
    assert_eq!(
        model.reversed_actions,
        HashSet::from([
            ComposerAction::Bold,
            ComposerAction::Italic,
            ComposerAction::Underline,
        ]),
    )
}

#[test]
fn updating_model_updates_disabled_actions() {
    let mut model = cm("|");
    assert_eq!(
        model.disabled_actions,
        HashSet::from([
            ComposerAction::Undo,
            ComposerAction::Redo,
            ComposerAction::Indent,
            ComposerAction::UnIndent
        ]),
    );
    replace_text(&mut model, "a");
    model.select(Location::from(0), Location::from(1));
    model.format(InlineFormatType::Bold);
    assert_eq!(
        model.disabled_actions,
        HashSet::from([
            ComposerAction::Redo,
            ComposerAction::Indent,
            ComposerAction::UnIndent
        ])
    );
    model.undo();
    assert_eq!(
        model.disabled_actions,
        HashSet::from([ComposerAction::Indent, ComposerAction::UnIndent])
    );
    model.redo();
    assert_eq!(
        model.disabled_actions,
        HashSet::from([
            ComposerAction::Redo,
            ComposerAction::Indent,
            ComposerAction::UnIndent
        ])
    );
    model.undo();
    model.undo();
    assert_eq!(
        model.disabled_actions,
        HashSet::from([
            ComposerAction::Undo,
            ComposerAction::Indent,
            ComposerAction::UnIndent
        ])
    );
}

#[test]
fn test_menu_updates_indent() {
    let model = cm("<ul><li>First item</li><li>{Second item}|</li></ul>");
    assert_eq!(
        model.disabled_actions,
        HashSet::from([
            ComposerAction::UnIndent,
            ComposerAction::Undo,
            ComposerAction::Redo
        ])
    );
}

#[test]
fn test_menu_updates_un_indent() {
    let model =
        cm("<ul><li>First item<ul><li>{Second item}|</li></ul></li></ul>");
    assert_eq!(
        model.disabled_actions,
        HashSet::from([
            ComposerAction::Indent,
            ComposerAction::Undo,
            ComposerAction::Redo
        ])
    );
}

fn replace_text(model: &mut ComposerModel<Utf16String>, new_text: &str) {
    model.replace_text(utf16(new_text));
}
