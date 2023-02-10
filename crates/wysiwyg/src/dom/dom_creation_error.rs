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

use core::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum DomCreationError {
    HtmlParseError(HtmlParseError),
    MarkdownParseError(MarkdownParseError),
}

#[derive(Debug, Eq, PartialEq)]
pub struct HtmlParseError {
    pub parse_errors: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MarkdownParseError {
    InvalidMarkdownError,
}

impl fmt::Display for MarkdownParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Self::InvalidMarkdownError => "unable to parse markdown",
        };
        write!(f, "{message}")
    }
}
