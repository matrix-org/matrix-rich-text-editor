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

use wysiwyg::{ComposerModel, TextUpdate};

#[test]
fn can_instantiate_a_model_and_call_methods() {
    let mut model = ComposerModel::new();
    model.replace_text("foo");
    model.select(cp(1), cp(2));

    let update = model.bold();

    if let TextUpdate::ReplaceAll(r) = update.text_update {
        assert_eq!(r.replacement_html, "f<strong>o</strong>o");
        assert_eq!(r.selection_start_codepoint.as_usize(), 1);
        assert_eq!(r.selection_end_codepoint.as_usize(), 2);
    } else {
        panic!("Expected to receive a ReplaceAll response");
    }
}

fn cp(value: usize) -> wysiwyg::CodepointLocation {
    wysiwyg::CodepointLocation::from(value)
}
