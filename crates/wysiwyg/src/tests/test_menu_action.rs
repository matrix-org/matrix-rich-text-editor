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
    tests::testutils_composer_model::cm, Location, MenuAction, PatternKey,
    SuggestionPattern,
};

// MenuAction computation tests.
#[test]
fn at_pattern_is_detected() {
    let model = cm("@alic|");
    assert_eq!(model.compute_menu_action(), sp(At, "alic", 0, 5),);
}

#[test]
fn at_pattern_is_not_detected_if_preceded_by_non_whitespace_char() {
    let model = cm("alice@matri|");
    assert_eq!(model.compute_menu_action(), MenuAction::None);
}

#[test]
fn empty_at_pattern_is_detected() {
    let model = cm("@|");
    assert_eq!(model.compute_menu_action(), sp(At, "", 0, 1));
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
fn at_pattern_is_not_detected_in_inline_code() {
    let model = cm("<code>@alic|</code>");
    assert_eq!(model.compute_menu_action(), MenuAction::None);
}

#[test]
fn at_pattern_is_detected_if_cursor_is_right_before() {
    let model = cm("|@alic");
    assert_eq!(model.compute_menu_action(), sp(At, "alic", 0, 5));
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

#[test]
fn slash_pattern_is_not_detected_if_not_at_the_beginning_of_dom() {
    let model = cm("abc /invi|");
    assert_eq!(model.compute_menu_action(), MenuAction::None);
}

// MenuAction update tests.
#[test]
fn at_pattern_is_updated_on_character_input() {
    let mut model = cm("|");
    assert_eq!(model.compute_menu_action(), MenuAction::None);
    let update = model.replace_text("@ali".into());
    assert_eq!(update.menu_action, sp(At, "ali", 0, 4));
    let update = model.replace_text("c".into());
    assert_eq!(update.menu_action, sp(At, "alic", 0, 5));
}

#[test]
fn at_pattern_is_updated_on_whitespace_input() {
    let mut model = cm("@alic|");
    let update = model.replace_text(" ".into());
    assert_eq!(update.menu_action, MenuAction::None);
}

#[test]
fn at_pattern_is_updated_upon_selection() {
    let mut model = cm("@alic abc|");
    let update = model.select(Location::from(5), Location::from(5));
    assert_eq!(update.menu_action, sp(At, "alic", 0, 5));
}

#[test]
fn at_pattern_is_updated_on_backspace() {
    let mut model = cm("@alic|");
    let update = model.backspace();
    assert_eq!(update.menu_action, sp(At, "ali", 0, 4));

    let mut model = cm("@|alic");
    let update = model.backspace();
    assert_eq!(update.menu_action, MenuAction::None);
}

#[test]
fn at_pattern_is_updated_on_delete() {
    let mut model = cm("@|alic");
    let update = model.delete();
    assert_eq!(update.menu_action, sp(At, "lic", 0, 4));

    let mut model = cm("|@alic");
    let update = model.delete();
    assert_eq!(update.menu_action, MenuAction::None);
}

#[test]
fn at_pattern_is_still_detected_after_moving_inside_structure_node() {
    let mut model = cm("@alic|");
    let update = model.ordered_list();
    assert_eq!(update.menu_action, sp(At, "alic", 0, 5));

    let mut model = cm("@alic|");
    let update = model.quote();
    assert_eq!(update.menu_action, sp(At, "alic", 0, 5));
}

#[test]
fn at_pattern_is_still_detected_after_applying_formatting() {
    let mut model = cm("{@alic}|");
    let update = model.bold();
    assert_eq!(update.menu_action, sp(At, "alic", 0, 5))
}

#[test]
fn at_pattern_is_not_detected_after_moving_in_code_block() {
    let mut model = cm("@alic|");
    let update = model.code_block();
    assert_eq!(update.menu_action, MenuAction::None);
}

#[test]
fn menu_action_retuns_keep_after_format_with_cursor() {
    let mut model = cm("@alic|");
    let update = model.bold();
    assert_eq!(update.menu_action, MenuAction::Keep);
    // Unbold
    let update = model.bold();
    assert_eq!(update.menu_action, MenuAction::Keep);
}

/// Short wrapper around [MenuAction::Suggestion(SuggestionPattern)].
fn sp(k: PatternKey, t: &str, s: usize, e: usize) -> MenuAction {
    MenuAction::Suggestion(SuggestionPattern {
        key: k,
        text: t.into(),
        start: s,
        end: e,
    })
}
