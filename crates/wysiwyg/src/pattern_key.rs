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

use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PatternKey {
    At,
    Hash,
    Slash,
    Custom(String),
}

impl PatternKey {
    pub(crate) fn is_static_pattern(&self) -> bool {
        matches!(self, Self::At | Self::Hash | Self::Slash)
    }

    pub(crate) fn from_string_and_suggestions(
        string: String,
        custom_suggestion_patterns: &HashSet<String>,
    ) -> Option<Self> {
        if custom_suggestion_patterns.contains(&string) {
            return Some(Self::Custom(string));
        }
        let Some(first_char) = string.chars().nth(0) else {
            return None;
        };
        match first_char {
            '\u{0040}' => Some(Self::At),
            '\u{0023}' => Some(Self::Hash),
            '\u{002F}' => Some(Self::Slash),
            _ => None,
        }
    }
}
