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

use widestring::Utf16String;

use crate::{ComposerModel, UnicodeString};

/// Short wrapper around [ComposerModel::from_example_format].
pub fn cm(text: &str) -> ComposerModel<Utf16String> {
    ComposerModel::<Utf16String>::from_example_format(text)
}

/// Short wrapper around [ComposerModel::to_example_format].
pub fn tx(model: &ComposerModel<Utf16String>) -> String {
    model.to_example_format()
}

pub(crate) fn restore_whitespace(text: &String) -> String {
    text.replace("&nbsp;", " ").replace("\u{A0}", " ")
}

pub(crate) fn restore_whitespace_u16(text: &Utf16String) -> Utf16String {
    Utf16String::from(restore_whitespace(&text.to_utf8()))
}
