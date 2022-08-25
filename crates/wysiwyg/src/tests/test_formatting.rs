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

use crate::tests::testutils_composer_model::{cm, tx};

use crate::{InlineFormatType, Location, ToRawText};

#[test]
fn selecting_and_bolding_multiple_times() {
    let mut model = cm("aabbcc|");
    model.select(Location::from(0), Location::from(2));
    model.format(InlineFormatType::Bold);
    model.select(Location::from(4), Location::from(6));
    model.format(InlineFormatType::Bold);
    assert_eq!(
        &model.state.dom.to_string(),
        "<strong>aa</strong>bb<strong>cc</strong>"
    );
    assert_eq!(&model.state.dom.to_raw_text(), "aabbcc");
}

#[test]
fn bolding_ascii_adds_strong_tags() {
    let mut model = cm("aa{bb}|cc");
    model.format(InlineFormatType::Bold);
    assert_eq!(tx(&model), "aa<strong>{bb}|</strong>cc");

    let mut model = cm("aa|{bb}cc");
    model.format(InlineFormatType::Bold);
    assert_eq!(tx(&model), "aa<strong>|{bb}</strong>cc");
}
