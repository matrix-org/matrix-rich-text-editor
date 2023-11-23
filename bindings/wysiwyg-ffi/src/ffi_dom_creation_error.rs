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

use std::{error::Error, fmt::Display};

#[derive(Debug, uniffi::Error)]
pub enum DomCreationError {
    HtmlParseError,
    MarkdownParseError,
}

impl Display for DomCreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            DomCreationError::HtmlParseError => {
                "could not create dom from html"
            }
            DomCreationError::MarkdownParseError => {
                "could not create dom from markdown"
            }
        })
    }
}

impl From<wysiwyg::DomCreationError> for DomCreationError {
    fn from(error: wysiwyg::DomCreationError) -> Self {
        match error {
            wysiwyg::DomCreationError::HtmlParseError(_) => {
                Self::HtmlParseError
            }
            wysiwyg::DomCreationError::MarkdownParseError(_) => {
                Self::MarkdownParseError
            }
        }
    }
}

impl From<DomCreationError> for wysiwyg::DomCreationError {
    fn from(_: DomCreationError) -> Self {
        unimplemented!("Error is not needed as input")
    }
}

impl Error for DomCreationError {}
