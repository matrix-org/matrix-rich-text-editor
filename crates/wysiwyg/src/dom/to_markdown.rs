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

use super::UnicodeString;
use bitflags::bitflags;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum MarkdownError<S>
where
    S: UnicodeString,
{
    UnknownContainerName(<S::Str as ToOwned>::Owned),
}

impl<S> Error for MarkdownError<S> where S: UnicodeString {}

impl<S> fmt::Display for MarkdownError<S>
where
    S: UnicodeString,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownContainerName(name) => {
                write!(formatter, "Unknown container name: `{:?}`", name)
            }
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
        let mut buf = S::default();
        self.fmt_markdown(&mut buf, &MarkdownOptions::empty())?;

        Ok(buf)
    }
}

bitflags! {
    pub struct MarkdownOptions: u8 {
        const IGNORE_LINE_BREAK = 0b0001;
    }
}
