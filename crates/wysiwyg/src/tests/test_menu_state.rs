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

use crate::tests::testutils_composer_model::cm;

use crate::{InlineFormatType, Location, ToolbarButton};

#[test]
fn creating_and_deleting_lists_updates_active_buttons() {
    let mut model = cm("|");
    model.ordered_list();
    assert_eq!(
        model.active_buttons,
        HashSet::from([ToolbarButton::OrderedList])
    );
    model.unordered_list();
    assert_eq!(
        model.active_buttons,
        HashSet::from([ToolbarButton::UnorderedList])
    );
    model.backspace();
    assert_eq!(model.active_buttons, HashSet::new());
}

#[test]
fn selecting_nested_nodes_updates_active_buttons() {
    let model = cm("<ul><li><b><i>{ab}|</i></b></li></ul>");
    assert_eq!(
        model.active_buttons,
        HashSet::from([
            ToolbarButton::UnorderedList,
            ToolbarButton::Bold,
            ToolbarButton::Italic,
        ]),
    );
}

#[test]
fn selecting_multiple_nodes_updates_active_buttons() {
    let model = cm("<ol><li>{ab</li><li><b>cd</b>}|</li></ol>");
    assert_eq!(
        model.active_buttons,
        HashSet::from([ToolbarButton::OrderedList])
    );
    let model = cm("<ol><li>{ab</li></ol>cd}|");
    assert_eq!(model.active_buttons, HashSet::new());

    let mut model = cm("<a href=\"https://matrix.org\">{link}|</a>ab");
    assert_eq!(model.active_buttons, HashSet::from([ToolbarButton::Link]));
    model.select(Location::from(2), Location::from(6));
    assert_eq!(model.active_buttons, HashSet::new());

    let mut model = cm("<del>{ab<em>cd}|</em></del>");
    assert_eq!(
        model.active_buttons,
        HashSet::from([ToolbarButton::StrikeThrough]),
    );
    model.select(Location::from(2), Location::from(4));
    assert_eq!(
        model.active_buttons,
        HashSet::from([ToolbarButton::Italic, ToolbarButton::StrikeThrough,]),
    )
}

#[test]
fn formatting_updates_active_buttons() {
    let mut model = cm("a{bc}|d");
    model.format(InlineFormatType::Bold);
    model.format(InlineFormatType::Italic);
    model.format(InlineFormatType::Underline);
    assert_eq!(
        model.active_buttons,
        HashSet::from([
            ToolbarButton::Bold,
            ToolbarButton::Italic,
            ToolbarButton::Underline,
        ]),
    )
}
