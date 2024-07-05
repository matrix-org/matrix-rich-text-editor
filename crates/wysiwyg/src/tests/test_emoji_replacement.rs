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
    tests::testutils_composer_model::tx, ComposerModel, MenuAction, PatternKey,
};

#[test]
fn can_do_plain_text_to_empji_replacement() {
    let mut model: ComposerModel<Utf16String> = ComposerModel::new();
    model.set_custom_suggestion_patterns(vec![":)".into()]);
    let update = model.replace_text("Hey That's great! :)".into());
    let MenuAction::Suggestion(suggestion) = update.menu_action else {
        panic!("No suggestion pattern found")
    };
    assert_eq!(suggestion.key, PatternKey::Custom(":)".into()),);
    model.replace_text_suggestion("🙂".into(), suggestion, false);

    assert_eq!(tx(&model), "Hey That's great! 🙂|");
}
