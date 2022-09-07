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

use crate::UnicodeString;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ListType {
    Ordered,
    Unordered,
}

impl ListType {
    pub(crate) fn tag(&self) -> &'static str {
        match self {
            ListType::Ordered => "ol",
            ListType::Unordered => "ul",
        }
    }
}

impl<S: UnicodeString> From<S> for ListType {
    fn from(value: S) -> Self {
        match value.to_utf8().as_str() {
            "ol" => ListType::Ordered,
            "ul" => ListType::Unordered,
            _ => {
                panic!("Unknown list type {}", value.to_utf8().as_str());
            }
        }
    }
}
