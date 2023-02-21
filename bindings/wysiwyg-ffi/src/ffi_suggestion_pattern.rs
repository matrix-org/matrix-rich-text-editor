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

use crate::PatternKey;

#[derive(Debug, PartialEq, Eq)]
pub struct SuggestionPattern {
    pub key: PatternKey,
    pub text: String,
    pub start: u32,
    pub end: u32,
}

impl From<wysiwyg::SuggestionPattern> for SuggestionPattern {
    fn from(inner: wysiwyg::SuggestionPattern) -> Self {
        Self {
            key: PatternKey::from(inner.key),
            text: inner.text,
            start: u32::try_from(inner.start).unwrap(),
            end: u32::try_from(inner.end).unwrap(),
        }
    }
}

impl From<SuggestionPattern> for wysiwyg::SuggestionPattern {
    fn from(pattern: SuggestionPattern) -> Self {
        Self {
            key: wysiwyg::PatternKey::from(pattern.key),
            text: pattern.text,
            start: usize::try_from(pattern.end).unwrap(),
            end: usize::try_from(pattern.end).unwrap(),
        }
    }
}
