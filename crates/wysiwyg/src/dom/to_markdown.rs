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

use std::{error::Error, fmt};
use unicode_string::UnicodeString;

#[derive(Debug)]
pub enum MarkdownError<S>
where
    S: UnicodeString,
{
    InvalidListItem(Option<S>),
}

impl<S> Error for MarkdownError<S> where S: UnicodeString {}

impl<S> fmt::Display for MarkdownError<S>
where
    S: UnicodeString,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidListItem(Some(node_name)) => write!(formatter, "A list expects a list item as immediate child, received `{node_name}`"),

            Self::InvalidListItem(None) => write!(formatter, "A list node expects a list item node as immediate child")
        }
    }
}

pub trait ToMarkdown<S>
where
    S: UnicodeString,
{
    fn fmt_markdown(
        &self,
        buffer: &mut S,
        options: &MarkdownOptions,
    ) -> Result<(), MarkdownError<S>>;

    fn to_markdown(&self) -> Result<S, MarkdownError<S>> {
        let mut buffer = S::default();
        self.fmt_markdown(&mut buffer, &MarkdownOptions::empty())?;

        Ok(buffer)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MarkdownOptions {
    bits: u8,
}

impl MarkdownOptions {
    pub const IGNORE_LINE_BREAK: Self = Self { bits: 0b0001 };

    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    /// Returns `true` if all of the flags in `other` are contained within `self`.
    pub const fn contains(&self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// Inserts the specified flags in-place.
    pub fn insert(&mut self, other: Self) {
        self.bits |= other.bits;
    }
}
