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

use crate::PatternKey::{At, Hash, Slash};
use crate::{
    tests::testutils_composer_model::cm, MenuAction, PatternKey,
    SuggestionPattern,
};

#[test]
fn at_pattern_is_detected() {
    let model = cm("@alic|");
    assert_eq!(model.compute_menu_action(), sp(At, "alic", 0, 5),)
}

#[test]
fn at_pattern_is_detected_after_text() {
    let model = cm("Hey @alic|");
    assert_eq!(model.compute_menu_action(), sp(At, "alic", 4, 9));
}

#[test]
fn at_pattern_is_detected_if_selection_is_entirely_inside() {
    let model = cm("Hey @a{li}|c");
    assert_eq!(model.compute_menu_action(), sp(At, "alic", 4, 9));
}

#[test]
fn at_pattern_is_not_detected_if_selection_spans_across_it() {
    let model = cm("{Hey @ali}|c");
    assert_eq!(model.compute_menu_action(), MenuAction::None);
}

#[test]
fn at_pattern_is_not_detected_after_whitespace() {
    let model = cm("@alic abc|");
    assert_eq!(model.compute_menu_action(), MenuAction::None);
}

#[test]
fn at_pattern_is_detected_in_formatting_node() {
    let model = cm("<em>Hey @bob|</em>");
    assert_eq!(model.compute_menu_action(), sp(At, "bob", 4, 8));
}

#[test]
fn at_pattern_is_detected_in_list() {
    let model = cm("<ol><li>@alic|</li></ol>");
    assert_eq!(model.compute_menu_action(), sp(At, "alic", 0, 5));
}

#[test]
fn at_pattern_is_detected_in_quote() {
    let model = cm("<blockquote><p>Hey @alic|</p></blockquote>");
    assert_eq!(model.compute_menu_action(), sp(At, "alic", 4, 9));
}

#[test]
fn at_pattern_is_not_detected_in_code_block() {
    let model = cm("<pre><code>@bob|</code></pre>");
    assert_eq!(model.compute_menu_action(), MenuAction::None);
}

#[test]
fn suggestion_applies_additional_offset_from_structure_nodes() {
    let model = cm("abc<ul><li>item</li><li>@alic|</li></ul>");
    assert_eq!(model.compute_menu_action(), sp(At, "alic", 9, 14));
}

#[test]
fn hash_pattern_is_detected() {
    let model = cm("#RichTex|");
    assert_eq!(model.compute_menu_action(), sp(Hash, "RichTex", 0, 8));
}

#[test]
fn slash_pattern_is_detected() {
    let model = cm("/invi|");
    assert_eq!(model.compute_menu_action(), sp(Slash, "invi", 0, 5));
}

fn sp(k: PatternKey, t: &str, s: usize, e: usize) -> MenuAction {
    MenuAction::Suggestion(SuggestionPattern {
        key: k,
        text: t.into(),
        start: s,
        end: e,
    })
}
